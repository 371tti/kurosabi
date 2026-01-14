use crate::connection::{Connection, NoneBody};
use crate::error::ErrorPare;
use crate::error::RouterError;
use crate::http::HttpStatusCode;
use crate::http::request::parse_range_header_value;
use crate::http::request::{RangeParse, RangeSpec};
use crate::{connection::ResponseReadyToSend, error::ConnectionResult};
use futures_io::{AsyncRead, AsyncWrite};
use std::ops::Range;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio_util::compat::TokioAsyncReadCompatExt;

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, NoneBody> {
    /// range はserver側で許可された範囲内で処理する
    #[inline]
    #[deprecated(
        note = "十分な検証ができていません。streamingメソッドで代替できます。"
    )]
    #[cfg(feature = "tokio-server")]
    pub async fn file_body(
        mut self,
        file_content: FileContentBuilder,
    ) -> ConnectionResult<Connection<C, R, W, ResponseReadyToSend>> {
        let mut file_content = match file_content.build().await {
            Ok(fc) => fc,
            Err(_) => {
                return Ok(self.set_status_code(HttpStatusCode::NotFound).no_body());
            },
        };
        let header_range = self.req.header_get("Range").await;
        let range = if file_content.force_range {
            file_content.default_range
        } else {
            match &header_range {
                Some(r) => {
                    match parse_range_header_value(r) {
                        RangeParse::Valid(RangeSpec::FromToInclusive { start, end }) => {
                            let range = file_content.default_range;
                            let start = start.max(range.start);
                            // HTTPのRangeヘッダはendがinclusiveなので+1する
                            let end = (end.saturating_add(1))
                                .min(range.end)
                                .min(start.saturating_add(file_content.max_size));
                            if start < end {
                                start..end
                            } else {
                                return Ok(self
                                    .set_status_code(HttpStatusCode::RangeNotSatisfiable)
                                    .no_body());
                            }
                        },
                        RangeParse::Valid(RangeSpec::From { start }) => {
                            let range = file_content.default_range;
                            let start = start.max(range.start);
                            let end = range.end.min(start + file_content.max_size);
                            if start < end {
                                start..end
                            } else {
                                return Ok(self
                                    .set_status_code(HttpStatusCode::RangeNotSatisfiable)
                                    .no_body());
                            }
                        },
                        RangeParse::Valid(RangeSpec::Suffix { len }) => {
                            let range = file_content.default_range;
                            let end = range.end;
                            let start = end
                                .saturating_sub(len.min(file_content.max_size))
                                .max(range.start);
                            if start < end {
                                start..end
                            } else {
                                return Ok(self
                                    .set_status_code(HttpStatusCode::RangeNotSatisfiable)
                                    .no_body());
                            }
                        },
                        RangeParse::Invalid => {
                            // return Ok(self.set_status_code(HttpStatusCode::RangeNotSatisfiable).no_body())
                            file_content.default_range
                        },
                    }
                },
                None => file_content.default_range,
            }
        };

        self.res.header_add("Content-Type", file_content.mime_type);
        match file_content.disposition {
            ContentDisposition::Inline => {
                self.res.header_add("Content-Disposition", "inline");
            },
            ContentDisposition::Attachment => {
                self.res.header_add("Content-Disposition", "attachment");
            },
            ContentDisposition::AttachmentWithFilename(fname) => {
                let disp_value = format!("attachment; filename=\"{}\"", fname);
                self.res.header_add("Content-Disposition", disp_value);
            },
        }
        if !file_content.force_range {
            self.res.header_add("Accept-Ranges", "bytes");
        }
        let start = range.start;
        let end = range.end;
        let size = end - start;
        let is_partial = size < file_content.full_size;
        let reader = file_content
            .file
            .seek(std::io::SeekFrom::Start(start))
            .await
            .and_then(|_| Ok(file_content.file.take(size).compat()));
        match reader {
            Ok(r) => {
                if is_partial {
                    self.res.header_add(
                        "Content-Range",
                        format!("bytes {}-{}/{}", start, end - 1, file_content.full_size),
                    );
                    self.set_status_code(HttpStatusCode::PartialContent)
                        .streaming_unchunked(r, size)
                        .await
                } else {
                    self.set_status_code(HttpStatusCode::OK)
                        .streaming_unchunked(r, size)
                        .await
                }
            },
            Err(e) => {
                let conn = Connection {
                    c: self.c,
                    req: self.req,
                    res: self.res,
                    phantom: std::marker::PhantomData,
                };
                return Err(ErrorPare {
                    router_error: RouterError::IoError(e),
                    connection: conn,
                });
            },
        }
    }
}

pub struct FileContentBuilder {
    path: std::path::PathBuf,
    content_type: ContentType,
    content_range: ContentRange,
    content_disposition: ContentDisposition,
}

pub struct FileContent {
    pub file: File,
    pub mime_type: String,
    pub full_size: u64,
    /// range ヘッダがない場合に使える範囲
    pub default_range: Range<u64>,
    pub max_size: u64,
    pub disposition: ContentDisposition,
    pub is_partly: bool,
    pub force_range: bool,
}

pub enum ContentType {
    Guess,
    Custom(String),
}

pub enum ContentRange {
    /// follow range header or full size
    Auto,
    /// auto with limit
    /// start from range header or 0
    /// end is min(limit, full size)
    AutoWithLimit(u64),
    /// force start and end
    /// not allow range header
    StartEnd(u64, u64),
}

pub enum ContentDisposition {
    Inline,
    Attachment,
    AttachmentWithFilename(String),
}

impl FileContentBuilder {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();
        let disposition = path
            .file_name()
            .and_then(|fname| fname.to_str())
            .map(|fname_str| ContentDisposition::AttachmentWithFilename(fname_str.to_string()))
            .unwrap_or(ContentDisposition::Attachment);
        FileContentBuilder {
            path: path.to_path_buf(),
            content_type: ContentType::Guess,
            content_range: ContentRange::Auto,
            content_disposition: disposition,
        }
    }

    pub fn name<S>(mut self, file_name: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.content_disposition = match file_name {
            Some(name) => ContentDisposition::AttachmentWithFilename(name.into()),
            None => ContentDisposition::Attachment,
        };
        self
    }

    pub fn inline(mut self) -> Self {
        self.content_disposition = ContentDisposition::Inline;
        self
    }

    pub fn content_type(mut self, content_type: ContentType) -> Self {
        self.content_type = content_type;
        self
    }

    pub fn limit_range(mut self, limit: u64) -> Self {
        self.content_range = ContentRange::AutoWithLimit(limit);
        self
    }

    pub fn range(mut self, content_range: ContentRange) -> Self {
        self.content_range = content_range;
        self
    }

    pub async fn build(self) -> std::io::Result<FileContent> {
        let file = File::open(&self.path).await?;
        let metadata = file.metadata().await?;
        let mime_type = match &self.content_type {
            ContentType::Guess => mime_guess::from_path(&self.path)
                .first_or_octet_stream()
                .essence_str()
                .to_string(),
            ContentType::Custom(s) => s.clone(),
        };
        let full_size = metadata.len();
        let mut is_partly = false;
        let mut default_range = 0..full_size;
        let mut max_size = full_size;
        let mut force_range = false;
        match &self.content_range {
            ContentRange::Auto => {},
            ContentRange::AutoWithLimit(limit) => {
                let limit = if limit < &full_size {
                    is_partly = true;
                    *limit
                } else {
                    full_size
                };
                max_size = limit;
            },
            ContentRange::StartEnd(start, end) => {
                let (start, end) = if start < end { (*start, *end) } else { (*end, *start) };
                let safe_start = (start).min(full_size);
                let safe_end = (end).min(full_size);
                default_range = safe_start..safe_end;
                is_partly = safe_start != 0 || safe_end != full_size;
                force_range = true;
            },
        };
        Ok(FileContent {
            disposition: self.content_disposition,
            mime_type,
            file,
            full_size,
            default_range,
            max_size,
            force_range,
            is_partly,
        })
    }
}
