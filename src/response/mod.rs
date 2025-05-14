use std::ffi::OsStr;
use std::io::SeekFrom;
use std::path::Path;
use std::pin::Pin;

use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite};
use tokio_util::io::ReaderStream;

use tokio::io::AsyncWriteExt;
use mime_guess::from_path;
use async_compression::tokio::write::BrotliEncoder;

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
    pub compress_enabled: bool,
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
    ///
    /// * `path` … 返したいファイル
    /// * Range ヘッダ対応（`bytes=start-end` / `start-` / `-suffix` すべてOK）
    pub async fn file<P: AsRef<Path>>(
        &mut self,
        req: &Req,
        path: P,
        inline: bool,
    ) -> Result<&mut Self, HttpError> {
        let path = path.as_ref();

        /* ---------- ファイル open & metadata ---------- */
        let mut file = File::open(path)
            .await
            .map_err(|_| HttpError::NotFound)?;
        let meta = file
            .metadata()
            .await
            .map_err(|_| HttpError::InternalServerError("metadata failed".into()))?;
        let size = meta.len();

        /* ---------- Content-Type 推定 ---------------- */
        let mime = from_path(path);
        let mime_type = mime.first_or_octet_stream();
        let ctype = mime_type.essence_str();
        self.header.set("Content-Type", ctype);

        /* ---------- Content-Disposition -------------- */
        if let Some(fname) = path.file_name().and_then(OsStr::to_str) {
            self.header
                .set("Content-Disposition", &format!("{}; filename=\"{}\"", if inline { "inline" } else { "attachment" }, fname));
        }

        /* ---------- Range 解析 ----------------------- */
        if let Some(range_raw) = req.header.get("Range") {
            // bytes=START-END / START- / -SUFFIX
            let err = || HttpError::BadRequest("Invalid Range".into());
            let bytes_part = range_raw
                .strip_prefix("bytes=")
                .ok_or_else(err)?;

            let mut start = 0;
            let mut end = size - 1;

            match bytes_part.split_once('-') {
                Some((s, e)) => {
                    if !s.is_empty() { start = s.parse().map_err(|_| err())?; }
                    if !e.is_empty() { end = e.parse().map_err(|_| err())?; }
                    if e.is_empty() && !s.is_empty() {
                        // case START-  (末尾まで)
                        end = size - 1;
                    }
                    if s.is_empty() && !e.is_empty() {
                        // case -SUFFIX (末尾 SUFFIX byte)
                        let suffix: u64 = e.parse().map_err(|_| err())?;
                        start = size.saturating_sub(suffix);
                        end = size - 1;
                    }
                }
                None => return Err(err()),
            }
            if start > end || end >= size {
                return Err(HttpError::RangeNotSatisfiable);
            }

            /* ---- range 転送 ---- */
            let len = end - start + 1;
            file.seek(SeekFrom::Start(start))
                .await
                .map_err(|_| HttpError::InternalServerError("seek failed".into()))?;
            self.body = Body::Stream(Box::pin(file.take(len)));
            self.code = 206; // Partial Content
            self.header.set("Content-Length", &len.to_string());
            self.header
                .set("Content-Range", &format!("bytes {}-{}/{}", start, end, size));
        } else {
            /* ---- 全量転送 (ReaderStream) ---------------- */
            let stream = ReaderStream::new(file);
            let stream_reader = tokio_util::io::StreamReader::new(stream);
            self.body = Body::Stream(Box::pin(stream_reader));
            self.header.set("Content-Length", &size.to_string());
        }

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
            compress_enabled: true,
        }
    }

    #[inline]
    pub fn decide_compression(&mut self, req: &Req) -> Compression {
        if self.compress_enabled == false {
            return Compression::NotCompressed;
        }
        
        // Accept-Encoding ヘッダを取得
        if let Some(encoding_list) = req.header.get_accept_encoding_vec() {
            if encoding_list.contains(&"br") {
                return Compression::BrOptimal;
            }
        }
        return Compression::NotCompressed;
    }

    #[inline]
    pub async fn flush(&mut self, req: &mut Req) -> Result<(), KurosabiError> {
        self.header.set("Server", "Kurosabi");
        let compression = self.decide_compression(req);
        self.body.compress(&mut self.header, compression).await;
        let writer = req.connection.writer();
        writer.write_all(format!("HTTP/1.1 {}\r\n", self.code).as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
        for (key, value) in &self.header.headers {
            writer.write_all(format!("{}: {}\r\n", key, value).as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
        }
        writer.write_all(b"\r\n").await.map_err(|e| KurosabiError::IoError(e))?;

        self.body.compress_to_stream(Compression::NotCompressed, writer).await?;

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

pub enum Compression {
    NotCompressed,
    BrOptimal,
    BrMid,
    BrLow,
    BrHi,
}

impl Body {
    #[inline]
    pub fn size(&self) -> usize {
        match self {
            Body::Empty => 0,
            Body::Text(text) => text.len(),
            Body::Binary(data) => data.len(),
            Body::Stream(_stream) => {
                // ストリームのサイズは不明
                // ここでは0を返す -> 非圧縮
                0
            }
        }
    }

    #[inline]
    pub async fn compress(
        &mut self,
        header: &mut Header,
        encoding: Compression,
    ) {
        match encoding {
            Compression::BrMid | Compression::BrLow | Compression::BrHi | Compression::BrOptimal => {
                // Brotli圧縮を行う
                let level = match encoding {
                    Compression::BrMid => 5,
                    Compression::BrLow => 1,
                    Compression::BrHi => 11,
                    Compression::BrOptimal => {
                        // 最適なレベルを選択
                        let size: usize = self.size();
                        let pow_2_size = size.checked_mul(size).unwrap_or(usize::MAX).max(1);
                        let level = (pow_2_size / 102400000).min(11);
                        println!("pow_2_size: {}, level: {}", pow_2_size, level);
                        if level == 0 {
                            return;
                        }
                        level
                    }
                    _ => unreachable!(),
                };
                let mut encoder = BrotliEncoder::with_quality(Vec::new(), async_compression::Level::Precise(level as i32));

                match self {
                    Body::Text(text) => {
                        encoder.write_all(text.as_bytes()).await.ok();
                        encoder.shutdown().await.ok();
                        let compressed = encoder.into_inner();
                        header.set("Content-Encoding", "br");
                        header.del("Content-Length");
                        header.set("Content-Length", &compressed.len().to_string());
                        header.set("X-Compression", &level.to_string());
                        *self = Body::Binary(compressed);
                    }
                    Body::Binary(data) => {
                        encoder.write_all(data).await.ok();
                        encoder.shutdown().await.ok();
                        let compressed = encoder.into_inner();
                        header.set("Content-Encoding", "br");
                        header.del("Content-Length");
                        header.set("Content-Length", &compressed.len().to_string());
                        header.set("X-Compression", &level.to_string());
                        *self = Body::Binary(compressed);
                    }
                    _ => {}
                }
            }
            _ => { }
        }
    }

    #[inline]
    pub async fn compress_to_stream<W>(
        &mut self,
        encoding: Compression,
        writer: &mut W,
    ) -> Result<(), KurosabiError>
    where
        W: AsyncWrite + Unpin,
    {
        match encoding {
            Compression::NotCompressed => {
                // 圧縮しない場合はそのまま書き込み
                match self {
                    Body::Text(text) => {
                        writer.write_all(text.as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
                    }
                    Body::Binary(data) => {
                        writer.write_all(data).await.map_err(|e| KurosabiError::IoError(e))?;
                    }
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
                    _ => {}
                }
                Ok(())
            }
            Compression::BrMid | Compression::BrLow | Compression::BrHi | Compression::BrOptimal => {
                // Brotli圧縮を行う
                let level = match encoding {
                    Compression::BrMid => 5,
                    Compression::BrLow => 1,
                    Compression::BrHi => 11,
                    Compression::BrOptimal => {
                        // 最適なレベルを選択
                        // 感覚的に大体だした2次関数ね
                        let size: usize = self.size();
                        let pow_2_size = size.checked_next_power_of_two().unwrap_or(usize::MAX).max(1);
                        let level = ((pow_2_size / 89600000) + 1).max(11);
                        level
                    }
                    _ => unreachable!(),
                };
                let mut encoder = BrotliEncoder::with_quality(writer, async_compression::Level::Precise(level as i32));

                match self {
                    Body::Text(text) => {
                        // テキストデータを圧縮して書き込み
                        encoder.write_all(text.as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
                    }
                    Body::Binary(data) => {
                        // バイナリデータを圧縮して書き込み
                        encoder.write_all(data).await.map_err(|e| KurosabiError::IoError(e))?;
                    }
                    Body::Stream(stream) => {
                        // ストリームデータを圧縮して書き込み
                        let mut reader = tokio::io::BufReader::new(stream);
                        let mut buffer = [0; 8192];
                        loop {
                            let n = reader.read(&mut buffer).await.map_err(|e| KurosabiError::IoError(e))?;
                            if n == 0 {
                                break;
                            }
                            encoder.write_all(&buffer[..n]).await.map_err(|e| KurosabiError::IoError(e))?;
                        }
                    }
                    _ => {}
                }
        
                // 圧縮ストリームを終了
                encoder.shutdown().await.map_err(|e| KurosabiError::IoError(e))?;
                Ok(())
            }
        }
    }
}
