use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};

use tokio::{io::BufWriter, net::TcpStream};
use tokio::io::AsyncWriteExt;

use crate::error::HttpError;
use crate::request::Req;
use crate::{error::KurosabiError, utils::header::Header};

pub struct Res {
    pub code: u16,
    pub header: Header,
    pub body: Body,
}

impl Res {
    pub fn text(mut self, text: &str) -> Self {
        self.header.set("Content-Type", "text/plain");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self // return self to allow method chaining
    }

    pub fn html(mut self, text: &str) -> Self {
        self.header.set("Content-Type", "text/html");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self // return self to allow method chaining
    }

    pub fn json(mut self, text: &str) -> Self {
        self.header.set("Content-Type", "application/json");
        self.header.set("Content-Length", &text.len().to_string());
        self.body = Body::Text(text.to_string());
        self // return self to allow method chaining
    }

    pub fn stream(mut self, stream: Pin<Box<dyn AsyncRead + Send + Sync>>) -> Self {
        self.body = Body::Stream(stream);
        self // return self to allow method chaining
    }

    pub async fn file<'a>(mut self, req: &Req<'a>, file: &std::path::PathBuf) -> Result<Self, HttpError> {
        self.header.set("Content-Type", "application/octet-stream");
        let metadata = file.metadata().map_err(|_| HttpError::InternalServerError)?;
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
            .map_err(|_| HttpError::InternalServerError)?;
        let metadata = f.metadata()
            .await
            .map_err(|_| HttpError::InternalServerError)?;
        let length = range.1 - range.0 + 1;
        f.seek(tokio::io::SeekFrom::Start(range.0))
            .await
            .map_err(|_| HttpError::InternalServerError)?;
        self.header.set("Content-Length", &length.to_string());
        self.header.set("Content-Range", &format!("bytes {}-{}/{}", range.0, range.1, metadata.len()));
        self.body = Body::Stream(Box::pin(f.take(length)));
        Ok(self)
    }
}

impl Res {
    pub fn set_cookie(&mut self, key: &str, value: &str) {
        self.header.set("Set-Cookie", &format!("{}={}", key, value));
    }

    pub fn del_cookie(&mut self, key: &str) {
        let indexs = self.header.indexs("Set-Cookie");
        for i in indexs {
            let (_, v) = self.header.headers[i].clone();
            if v.starts_with(&format!("{}=", key)) {
                self.header.index_del(i);
            }
        }
    }

    pub fn set_status(&mut self, code: u16) {
        self.code = code;
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.header.set(key, value);
    }

    pub fn del_header(&mut self, key: &str) {
        self.header.del(key);
    }
}

impl Res {
    pub fn new() -> Res {
        Res {
            code: 200,
            header: Header::new(),
            body: Body::Empty,
        }
    }

    pub async fn write_out_connection(&mut self, conn: &mut TcpStream) -> Result<(), KurosabiError> {
        let mut writer = BufWriter::new(conn);
        writer.write_all(format!("HTTP/1.1 {}\r\n", self.code).as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
        for (key, value) in &self.header.headers {
            writer.write_all(format!("{}: {}\r\n", key, value).as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
        }
        writer.write_all(b"\r\n").await.map_err(|e| KurosabiError::IoError(e))?;
        
        match &mut self.body {
            Body::Empty => (),
            Body::Text(text) => writer.write_all(text.as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?,
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
    Stream(Pin<Box<dyn AsyncRead + Send + Sync>>),
}