use std::{ffi::OsStr, io::SeekFrom, path::Path, pin::Pin};

use async_compression::tokio::write::BrotliEncoder;
use mime_guess::from_path;
use tokio::{fs::File, io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter}, net::tcp::OwnedWriteHalf};
use tokio_util::io::ReaderStream;

use crate::{error::{HttpError, KurosabiError}, kurosabi::Context, utils::header::Header};

use super::Res;

/// レスポンス構築するやつ
impl Res {
    /// テキストレスポンス
    #[inline]
    pub fn text(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "text/plain; charset=utf-8");
        self.header.set("Content-Length", &text.as_bytes().len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// HTMLレスポンス
    #[inline]
    pub fn html(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "text/html");
        self.header.set("Content-Length", &text.as_bytes().len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// XMLレスポンス
    #[inline]
    pub fn xml(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "application/xml");
        self.header.set("Content-Length", &text.as_bytes().len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// JSレスポンス
    #[inline]
    pub fn js(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "application/javascript");
        self.header.set("Content-Length", &text.as_bytes().len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// CSSレスポンス
    #[inline]
    pub fn css(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "text/css");
        self.header.set("Content-Length", &text.as_bytes().len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// CSVレスポンス
    #[inline]
    pub fn csv(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "text/csv");
        self.header.set("Content-Length", &text.as_bytes().len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// JSONレスポンス
    #[inline]
    pub fn json(&mut self, text: &str) -> &mut Self {
        self.header.set("Content-Type", "application/json");
        self.header.set("Content-Length", &text.as_bytes().len().to_string());
        self.body = Body::Text(text.to_string());
        self
    }

    /// JSONレスポンス
    #[inline]
    pub fn json_value(&mut self, value: &serde_json::Value) -> &mut Self {
        self.header.set("Content-Type", "application/json");
        let text = serde_json::to_string(value).expect("json to text");
        self.header.set("Content-Length", &text.as_bytes().len().to_string());
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
    /// AsyncReadを実装したストリームを渡してね
    /// context_length が必要、 不明な場合は .chunked_stream() を推奨します
    #[inline]
    pub fn stream(&mut self, stream: Pin<Box<dyn AsyncRead + Send + Sync>>, buffer_size: usize, content_length: usize) -> &mut Self {
        self.header.set("Content-Length", &content_length.to_string());
        self.body = Body::Stream(stream, buffer_size);
        self
    }

    /// チャンクドストリームレスポンス
    /// AsyncReadを実装したストリームを渡してね
    #[inline]
    pub fn chunked_stream(&mut self, stream: Pin<Box<dyn AsyncRead + Send + Sync>>, buffer_size: usize) -> &mut Self {
        self.body = Body::ChunkedStream(stream, buffer_size);
        self
    }

    /// ファイルレスポンス
    ///
    /// * `path` … 返したいファイル
    /// * Range ヘッダ対応（`bytes=start-end` / `start-` / `-suffix` すべてOK）
    pub async fn file<P: AsRef<Path>, C>(
        mut context: Context<C>,
        path: P,
        inline: bool,
        file_name: Option<&str>
    ) -> Result<Context<C>, HttpError> {
        const DEFAULT_BUFFER_SIZE: usize = 16384; // デフォルトのバッファサイズ
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
        context.res.header.set("Content-Type", ctype);

        /* ---------- Content-Disposition -------------- */
        if let Some(fname) = file_name.or(path.file_name().and_then(OsStr::to_str)) {
            context.res.header
                .set("Content-Disposition", &format!("{}; filename=\"{}\"", if inline { "inline" } else { "attachment" }, fname));
        }

        /* ---------- Range 解析 ----------------------- */
        if let Some(range_raw) = context.req.header.get("Range") {
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
            context.res.body = Body::Stream(Box::pin(file.take(len)), DEFAULT_BUFFER_SIZE);
            context.res.code = 206; // Partial Content
            context.res.header.set("Content-Length", &len.to_string());
            context.res.header
                .set("Content-Range", &format!("bytes {}-{}/{}", start, end, size));
        } else {
            /* ---- 全量転送 (ReaderStream) ---------------- */
            let stream = ReaderStream::new(file);
            let stream_reader = tokio_util::io::StreamReader::new(stream);
            context.res.body = Body::Stream(Box::pin(stream_reader), DEFAULT_BUFFER_SIZE);
            context.res.header.set("Content-Length", &size.to_string());
        }


        context.res.header.set("Accept-Ranges", "bytes");

        Ok(context)
    }
}

pub enum Body {
    Empty,
    Text(String),
    Binary(Vec<u8>),
    Stream(Pin<Box<dyn AsyncRead + Send + Sync>>, usize),
    ChunkedStream(Pin<Box<dyn AsyncRead + Send + Sync>>, usize),
}

pub enum CompressionConfig {
    None,
    Optimal,
    Mid,
    Low,
    Hi,
}

pub enum Compression {
    NotCompressed,
    BrOptimal,
    BrMid,
    BrLow,
    BrHi,
}

impl Body {
    /// 圧縮率自動判定中
    #[inline]
    pub fn size(&self) -> usize {
        match self {
            Body::Empty => 0,
            Body::Text(text) => text.as_bytes().len(),
            Body::Binary(data) => data.len(),
            Body::Stream(_stream, _buffer_size) => {
                // ストリームのサイズは不明
                // ここでは0を返す -> 非圧縮
                0
            },
            Body::ChunkedStream(_stream, _buffer_size) => {
                // ストリームのサイズは不明
                // ここでは0を返す -> 非圧縮
                0
            }
        }
    }

    /// 圧縮を行う
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
                        header.replace("Content-Length", &compressed.len().to_string());
                        header.set("X-Compression", &level.to_string());
                        *self = Body::Binary(compressed);
                    }
                    Body::Binary(data) => {
                        encoder.write_all(data).await.ok();
                        encoder.shutdown().await.ok();
                        let compressed = encoder.into_inner();
                        header.set("Content-Encoding", "br");
                        header.replace("Content-Length", &compressed.len().to_string());
                        header.set("X-Compression", &level.to_string());
                        *self = Body::Binary(compressed);
                    }
                    _ => {
                        // empty stream chunkedstream は圧縮できない
                    }
                }
            }
            Compression::NotCompressed => {
                // 圧縮しない場合は何もしない
            }
        }
    }

    /// デフォルトヘッダを書き込む ないとエラーになる系
    #[inline]
    pub async fn write_default_headers(&self, header: &mut Header) {
        match self {
            Body::ChunkedStream(_, _) => {
                header.set("Transfer-Encoding", "chunked");
            }
            Body::Empty => {
                header.set("Content-Length", "0");
            }
            _ => {
                // 他は特に何もしないでいいと思う？
            }
        }
    }

    /// 書き込みと圧縮を同時に行う
    #[inline]
    pub async fn write_with_compression(
        self,
        encoding: Compression,
        writer: &mut BufWriter<OwnedWriteHalf>,
    ) -> Result<(), KurosabiError>
    {
        match encoding {
            Compression::NotCompressed => {
                // 圧縮しない場合はそのまま書き込み
                match self {
                    Body::Text(text) => {
                        writer.write_all(text.as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
                    }
                    Body::Binary(data) => {
                        writer.write_all(&data).await.map_err(|e| KurosabiError::IoError(e))?;
                    }
                    Body::Stream(mut stream, buffer_size) => {
                        let mut buffer = vec![0; buffer_size]; // Use a buffer of the specified size
                        let writer = writer.get_mut();
                        loop {
                            let n = stream.read(&mut buffer).await.map_err(|e| KurosabiError::IoError(e))?;
                            if n == 0 {
                                break; // ストリームの終端
                            }
                            writer.write_all(&buffer[..n]).await.map_err(|e| KurosabiError::IoError(e))?;
                            writer.flush().await.map_err(|e| KurosabiError::IoError(e))?;
                        }
                    },
                    Body::ChunkedStream(mut stream, buffer_size) => {
                        let mut buffer = vec![0; buffer_size]; // Use a buffer of the specified size
                        let writer = writer.get_mut();
                        loop {
                            let n = stream.read(&mut buffer).await.map_err(|e| KurosabiError::IoError(e))?;
                            if n == 0 {
                                writer.write_all(b"0\r\n\r\n").await.map_err(|e| KurosabiError::IoError(e))?;
                                writer.flush().await.map_err(|e| KurosabiError::IoError(e))?;
                                break; // ストリームの終端
                            }
                            // チャンクサイズを書き込む
                            let size_line = format!("{:X}\r\n", n);
                            writer.write_all(size_line.as_bytes()).await.map_err(|e| KurosabiError::IoError(e))?;
                            // データを書き込む
                            writer.write_all(&buffer[..n]).await.map_err(|e| KurosabiError::IoError(e))?;
                            writer.write_all(b"\r\n").await.map_err(|e| KurosabiError::IoError(e))?;
                            writer.flush().await.map_err(|e| KurosabiError::IoError(e))?;
                        }
                        // 最後のチャンクを書き込む
                    },
                    Body::Empty => {
                        // 何もないのでなにもしない
                    }
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
                        encoder.write_all(&data).await.map_err(|e| KurosabiError::IoError(e))?;
                    }
                    Body::Stream(stream, buffer_size) => {
                        // ストリームデータを圧縮して書き込み
                        let mut reader = tokio::io::BufReader::new(stream);
                        let mut buffer = vec![0; buffer_size];
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