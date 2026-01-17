use crate::connection::{Connection, NoneBody};
use crate::error::ErrorPare;
use crate::error::RouterError;
use crate::http::HttpStatusCode;
use crate::http::request::parse_range_header_value;
use crate::http::request::{RangeParse, RangeSpec};
use crate::utils::url_decode_fast;
use crate::{connection::ResponseReadyToSend, error::ConnectionResult};
use futures_io::{AsyncRead, AsyncWrite};
use mime_guess::mime;
use std::marker::PhantomData;
use std::ops::Range;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio_util::compat::TokioAsyncReadCompatExt;

impl<C, R: AsyncRead + Unpin + 'static, W: AsyncWrite + Unpin + 'static> Connection<C, R, W, NoneBody> {
    /// ファイルをレスポンスボディとして送信する
    /// rangeヘッダを考慮します
    #[inline]
    #[deprecated(note = "十分な検証ができていません。streamingメソッドで代替できます。")]
    #[cfg(feature = "tokio-server")]
    pub async fn file_body(
        mut self,
        file_content: FileContentBuilder<FileContentBuilderReady>,
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

pub trait FileContentBuilderState {}
pub struct FileContentBuilderInit;
impl FileContentBuilderState for FileContentBuilderInit {}
pub struct FileContentBuilderReady;
impl FileContentBuilderState for FileContentBuilderReady {}

/// ファイルレスポンスのビルダ
pub struct FileContentBuilder<S = FileContentBuilderInit> {
    base: std::path::PathBuf,
    path: std::path::PathBuf,
    content_type: ContentType,
    content_range: ContentRange,
    content_disposition: ContentDisposition,
    to_safe: bool,
    phantom: PhantomData<S>,
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

impl FileContentBuilder<FileContentBuilderInit> {
    /// ベースパスを指定
    pub fn base<P>(base: P) -> Self
    where
        P: AsRef<std::path::Path>,
    {
        let disposition = ContentDisposition::Attachment;
        FileContentBuilder {
            base: base.as_ref().to_path_buf(),
            path: PathBuf::new(),
            content_type: ContentType::Guess,
            content_range: ContentRange::Auto,
            content_disposition: disposition,
            to_safe: true,
            phantom: PhantomData,
        }
    }

    /// ベースパスからの相対パスを指定
    pub fn path<P>(self, path: P) -> FileContentBuilder<FileContentBuilderReady>
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
            base: self.base,
            path: path.to_path_buf(),
            content_type: self.content_type,
            content_range: self.content_range,
            content_disposition: disposition,
            to_safe: self.to_safe,
            phantom: PhantomData,
        }
    }

    pub fn path_url_segs(self, path: &[&str]) -> FileContentBuilder<FileContentBuilderReady> {
        let joined_path = path.join("/");
        let decoded_path = url_decode_fast(&joined_path);
        self.path(decoded_path.as_ref())
    }
}

impl FileContentBuilder<FileContentBuilderReady> {
    /// ファイル名を指定
    /// (inlineでなくなります)
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

    /// コンテントディスポジションをinlineに設定
    pub fn inline(mut self) -> Self {
        self.content_disposition = ContentDisposition::Inline;
        self
    }

    /// コンテントタイプを指定
    /// デフォルトはmime_guessで推測したタイプ
    pub fn content_type(mut self, content_type: ContentType) -> Self {
        self.content_type = content_type;
        self
    }

    /// コンテントレンジの最大値を指定
    /// rangeリクエスト時、送信するバイト数を制限する
    /// リソース保証用
    pub fn limit_range(mut self, limit: u64) -> Self {
        self.content_range = ContentRange::AutoWithLimit(limit);
        self
    }

    /// コンテントレンジの強制指定
    pub fn range(mut self, content_range: ContentRange) -> Self {
        self.content_range = content_range;
        self
    }

    /// パスの安全性チェックを無効化
    /// ディレクトリトラバーサル攻撃に対して脆弱になります
    pub fn allow_unsafe_path(mut self) -> Self {
        self.to_safe = false;
        self
    }

    pub(crate) fn safe_path_under(&mut self) -> std::io::Result<PathBuf> {
        // base を実体パス化（相対のままだと starts_with が壊れる）
        let base = self.base.canonicalize()?;

        // 絶対パス拒否
        if self.path.is_absolute() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "absolute path not allowed",
            ));
        }

        let joined = base.join(&self.path);
        let canon = joined.canonicalize()?;

        // base 配下チェック（symlink脱出も防ぐ）
        if !canon.starts_with(&base) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "path traversal detected",
            ));
        }

        // 正規化済みを保存（base も canonical に揃えると後工程も楽）
        self.base = base;
        Ok(canon)
    }

    /// ファイルの存在チェックを行う
    /// 無ければ Err としてディレクトリがあるか あればその相対パス一覧を返す
    pub async fn check_file_exists(mut self) -> Result<Self, Option<Vec<std::path::PathBuf>>> {
        let path = if self.to_safe {
            self.safe_path_under().map_err(|_| None)?
        } else {
            self.base.join(&self.path)
        };
        match tokio::fs::metadata(&path).await {
            Ok(meta) => {
                if meta.is_file() {
                    Ok(self)
                } else if meta.is_dir() {
                    let mut entries = tokio::fs::read_dir(&path).await.map_err(|_| None)?;
                    let mut paths = Vec::new();
                    while let Some(entry) = entries.next_entry().await.map_err(|_| None)? {
                        let path = entry.path();
                        if let Ok(rel_path) = path.strip_prefix(&self.base) {
                            paths.push(rel_path.to_path_buf());
                        }
                    }
                    Err(Some(paths))
                } else {
                    Err(None)
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(None),
            Err(_) => Err(None),
        }
    }

    pub(crate) async fn build(mut self) -> std::io::Result<FileContent> {
        self.path = if self.to_safe {
            self.safe_path_under()?
        } else {
            self.base.join(&self.path)
        };
        let mut file = File::open(&self.path).await?;
        let metadata = file.metadata().await?;
        let mime_type = match &self.content_type {
            ContentType::Guess => {
                let mime = mime_guess::from_path(&self.path).first_or_octet_stream();
                let fmt = match mime.type_() {
                    mime::TEXT
                    | mime::JSON
                    | mime::XML
                    | mime::JAVASCRIPT
                    | mime::WWW_FORM_URLENCODED
                    | mime::HTML
                    | mime::CSS => {
                        let mut buf = vec![0; 4096];
                        let n = file.read(&mut buf).await?;
                        buf.truncate(n);

                        file.seek(std::io::SeekFrom::Start(0)).await?;

                        let mut det = chardetng::EncodingDetector::new();
                        det.feed(&buf, true);
                        let encoding = det.guess(None, true);
                        let name = encoding.name(); // &'static str
                        match name {
                            // Unicode
                            "UTF-8" => Some("utf-8"),

                            // Japanese
                            "Shift_JIS" => Some("shift_jis"),
                            "EUC-JP" => Some("euc-jp"),
                            "ISO-2022-JP" => Some("iso-2022-jp"),

                            // Chinese
                            "GBK" => Some("gbk"),
                            "Big5" => Some("big5"),
                            "gb18030" => Some("gb18030"),
                            _ => None,
                        }
                    },
                    _ => None,
                };
                let mut mime = mime.essence_str().to_string();
                if let Some(fmt) = fmt {
                    mime.push_str("; charset=");
                    mime.push_str(fmt);
                }
                mime
            },
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
