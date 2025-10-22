use body::{Body, Compression, CompressionConfig};



use tokio::io::AsyncWriteExt;

use crate::request::Req;
use crate::{error::KurosabiError, utils::header::Header};

pub mod body;

pub struct Res {
    /// ステータスコード
    pub code: u16,
    /// ヘッダ
    pub header: Header,
    /// ボディ
    pub body: Body,
    pub compress_enabled: bool,
    pub compress_config: CompressionConfig,
}

impl Res {
    /// ステータスコードをセットする
    #[inline]
    pub fn set_status(&mut self, code: u16) {
        self.code = code;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.code = 200;
        self.header.clear();
        self.body = Body::Empty;
        self.compress_enabled = true;
        self.compress_config = CompressionConfig::Optimal;
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
            compress_config: CompressionConfig::Optimal,
        }
    }

    #[inline]
    pub fn decide_compression(&mut self, req: &Req) -> Compression {
        if self.compress_enabled == false {
            return Compression::NotCompressed;
        }
        
        // Accept-Encoding ヘッダを取得
        if let Some(encoding_list) = req.header.get_accept_encoding_vec() {
            match self.compress_config {
                CompressionConfig::Optimal => {
                    if encoding_list.contains(&"br") {
                        return Compression::BrOptimal;
                    }
                },
                CompressionConfig::Mid => {
                    if encoding_list.contains(&"br") {
                        return Compression::BrMid;
                    }
                },
                CompressionConfig::Low => {
                    if encoding_list.contains(&"br") {
                        return Compression::BrLow;
                    }
                },
                CompressionConfig::Hi => {
                    if encoding_list.contains(&"br") {
                        return Compression::BrHi;
                    }
                },
                CompressionConfig::None => {
                    return Compression::NotCompressed;
                },

            }
        }
        return Compression::NotCompressed;
    }

    #[inline]
    pub async fn flush(mut self, req: &mut Req) -> Result<(), KurosabiError> {
        let compression = self.decide_compression(req);
        self.body.compress(&mut self.header, compression).await;
        let writer = req.connection.writer();
        writer.write_all(format!("HTTP/1.1 {}\r\n", self.code).as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
        for (key, value) in &self.header.headers {
            writer.write_all(format!("{}: {}\r\n", key, value).as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
        }
        writer.write_all(b"\r\n").await.map_err(|e| KurosabiError::IoError(e))?;
        writer.flush().await.map_err(|e| KurosabiError::IoError(e))?;

        self.body.compress_to_stream(Compression::NotCompressed, writer).await?;

        writer.flush().await.map_err(|e| KurosabiError::IoError(e))?;
        Ok(())
    }
    
}