use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};

use tokio::io::AsyncWriteExt;

use crate::error::HttpError;
use crate::request::Req;
use crate::{error::KurosabiError, utils::header::Header};

pub struct Res {
    /// ステータスコード
    pub code: u16,
    /// ヘッダ
    pub header: Header,
    /// ボディ
    pub body: Body,
}

/// レスポンス構築するやつ
impl Res {
    /// テキストレスポンス
    #[inline]
    pub fn text(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "text/plain");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// HTMLレスポンス
    #[inline]
    pub fn html(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "text/html");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// XMLレスポンス
    #[inline]
    pub fn xml(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "application/xml");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// JSレスポンス
    #[inline]
    pub fn js(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "application/javascript");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// CSSレスポンス
    #[inline]
    pub fn css(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "text/css");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// CSVレスポンス
    #[inline]
    pub fn csv(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "text/csv");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// JSONレスポンス
    #[inline]
    pub fn json(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "application/json");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// JSONレスポンス
    #[inline]
    pub fn json_value(&mut self, value: &serde_json::Value) -> &mut Self {
        self.header.set("Content-Type", "application/json");
        let text = serde_json::to_string(value).unwrap();
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text);
        self
    }

    /// バイナリレスポンス
    #[inline]
    pub fn binary(&mut self, data: &[u8]) -> &mut Self {
        self.header.set("Content-Type", "application/octet-stream");
        self.header.set("Content-Length", &data.len().to_string());
        self.body = Body::Binary(data.to_vec());
        self
    }

    #[inline]
    pub fn data(&mut self, data: &[u8], content_type: &str) -> &mut Self {
        self.header.set("Content-Type", content_type);
        self.header.set("Content-Length", &data.len().to_string());
        self.body = Body::Binary(data.to_vec());
        self
    }

    /// ストリームレスポンス
    /// ストリームレスポンスは、AsyncReadを実装したストリームを指定する
    #[inline]
    pub fn stream(&mut self, stream: Pin<Box<dyn AsyncRead + Send + Sync>>) -> &mut Self {
        self.body = Body::Stream(stream);
        self
    }

    /// ファイルレスポンス
    /// ファイルレスポンスは、ファイルを指定する
    #[inline]
    pub async fn file(&mut self, req: &Req, file: &std::path::PathBuf) -> Result<&mut Self, HttpError> {
        self.header.set("Content-Type", "application/octet-stream");
        let metadata = file.metadata().map_err(|_| HttpError::InternalServerError("Failed to retrieve file metadata".to_string()))?;
        self.header.set("Content-Length", &metadata.len().to_string());
        self.header.set("Content-Disposition", &format!("attachment; filename={}", file.file_name().unwrap().to_str().unwrap()));
        let raw_range = req.header.get("Range");
        let range = if let Some(r) = raw_range {
            let r = r.split("=").collect::<Vec<&str>>();
            if r.len() != 2 {
                return Err(HttpError::BadRequest("Invalid Range Header".to_string()));
            }
            let r = r[1].split("-").collect::<Vec<&str>>();
            if r.len() != 2 {
                return Err(HttpError::BadRequest("Invalid Range Header".to_string()));
            }
            let start = r[0].parse::<u64>().map_err(|e| HttpError::BadRequest(e.to_string()))?;
            let end = r[1].parse::<u64>().map_err(|e| HttpError::BadRequest(e.to_string()))?;
            (start, end)
        } else {
            (0, metadata.len() - 1)
        };
        let mut f = tokio::fs::File::open(file)
            .await
            .map_err(|_| HttpError::InternalServerError("Failed to open file".to_string()))?;
        let metadata = f.metadata()
            .await
            .map_err(|_| HttpError::InternalServerError("Failed to retrieve file metadata".to_string()))?;
        let length = range.1 - range.0 + 1;
        f.seek(tokio::io::SeekFrom::Start(range.0))
            .await
            .map_err(|_| HttpError::InternalServerError("Failed to seek in file".to_string()))?;
        self.header.set("Content-Length", &length.to_string());
        self.header.set("Content-Range", &format!("bytes {}-{}/{}", range.0, range.1, metadata.len()));
        self.body = Body::Stream(Box::pin(f.take(length)));
        Ok(self)
    }
}

impl Res {
    /// ステータスコードをセットする
    #[inline]
    pub fn set_status(&mut self, code: u16) {
        self.code = code;
    }
}

impl Res {
    #[inline]
    pub fn new() -> Res {
        Res {
            code: 200,
            header: Header::new(),
            body: Body::Empty,
        }
    }

    #[inline]
    pub async fn flush(&mut self, req: &mut Req) -> Result<(), KurosabiError> {
        let writer = req.connection.writer();
        writer.write_all(format!("HTTP/1.1 {}\r\n", self.code).as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
        for (key, value) in &self.header.headers {
            writer.write_all(format!("{}: {}\r\n", key, value).as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
        }
        writer.write_all(b"\r\n").await.map_err(|e| KurosabiError::IoError(e))?;
        
        match &mut self.body {
            Body::Empty => (),
            Body::Text(text) => writer.write_all(text.as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?,
            Body::Binary(data) => writer.write_all(data).await.map_err(|e| KurosabiError::IoError(e))?,
            Body::Stream(stream) => {
                let mut reader = tokio::io::BufReader::new(stream);
                let mut buffer = [0; 8192];
                loop {
                    let n = reader.read(&mut buffer).await.map_err(|e| KurosabiError::IoError(e))?;
                    if n == 0 {
                        break;
                    }
                    writer.write_all(&buffer[..n]).await.map_err(|e| KurosabiError::IoError(e))?;
                }
            }
        }

        writer.flush().await.map_err(|e| KurosabiError::IoError(e))?;
        Ok(())
    }
    
}

pub enum Body {
    Empty,
    Text(String),
    Binary(Vec<u8>),
    Stream(Pin<Box<dyn AsyncRead + Send + Sync>>),
}