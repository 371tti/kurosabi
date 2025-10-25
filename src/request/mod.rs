pub mod path;

use path::Path;
use serde::de::DeserializeOwned;
use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader}, net::tcp::OwnedReadHalf};

use serde_json;
use crate::{error::{HttpError, KurosabiError}, server::TcpConnection, utils::header::Header};
use crate::utils::method::Method;

pub struct Req {
    /// HTTPメソッド
    /// GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH...
    pub method: Method,
    /// リクエストパス
    pub path: Path,
    /// HTTPヘッダ
    pub header: Header,
    /// HTTPバージョン
    pub version: String,
    /// 接続
    pub connection: TcpConnection,
}

impl Req {
    #[inline]
    pub fn new(connection: TcpConnection) -> Req {
        Req {
            method: Method::UNKNOWN("until parse".to_string()),
            path: Path::new(""),
            header: Header::new(),
            version: String::new(),
            connection: connection,
        }
    }

    #[inline]
    pub async fn wait_request(&mut self) -> Result<(), KurosabiError> {
        // 接続を待機し、何か受け取るまで待つ
        let reader = self.connection.reader();
        let mut buf = [0u8; 1];
        
        reader
            .get_mut()
            .peek(&mut buf)
            .await
            .map_err(KurosabiError::IoError)?;

        Ok(())
    }

    /// httpリクエストのヘッダを最低限パースする
    #[inline]
    pub async fn parse_headers(&mut self, max_header_size: usize) -> Result<(), KurosabiError> {
        let mut total_header_size = 0;
        let reader = self.connection.reader();
        let mut line_buf = String::with_capacity(1024);
        // Parse the request line first
        reader
            .read_line(&mut line_buf)
            .await
            .map_err(KurosabiError::IoError)?;
        total_header_size += line_buf.len();
        if total_header_size > max_header_size {
            return Err(KurosabiError::HeaderTooLarge);
        }
        let parts: Vec<&str> = line_buf.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err(KurosabiError::InvalidHttpHeader(line_buf));
        }

        let method = Method::from_str(parts[0]);
        let path = Path::new(parts[1]);
        let version = parts[2].to_string();
        let mut header = Header::new();

        loop {
            line_buf.clear();
            reader
                .read_line(&mut line_buf)
                .await
                .map_err(KurosabiError::IoError)?;
            total_header_size += line_buf.len();
            if total_header_size > max_header_size {
                return Err(KurosabiError::HeaderTooLarge);
            }
            let trimmed = line_buf.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some((key, value)) = trimmed.split_once(": ") {
                header.set(key, value);
            } else {
                return Err(KurosabiError::InvalidHttpHeader(line_buf));
            }
        }
        self.method = method;
        self.path = path;
        self.header = header;
        self.version = version;
        Ok(())
    }

    /// HTTPリクエストのボディをバイナリとして取得する
    /// Content-Length ヘッダーを使用して、指定されたサイズ分だけ読み込む
    #[inline]
    pub async fn body(&mut self) -> Result<Vec<u8>, HttpError> {
        // Content-Length ヘッダーから本文のサイズを取得
        let content_length = if let Some(cl) = self.header.get("CONTENT-LENGTH") {
            cl.parse::<usize>().map_err(|_| HttpError::InvalidLength(cl.to_string()))?
        } else {
            // Content-Lengthがない場合は空の Vec を返す
            return Ok(Vec::new());
        };
    
        let mut buf = vec![0u8; content_length];
        let reader = self.connection.reader();
        // 指定サイズ分だけ読み込む
        reader.read_exact(&mut buf).await.map_err(|e| HttpError::InternalServerError(e.to_string()))?;
        
        // 読み込んだバイト列を Vec<u8> として返す
        Ok(buf)
    }

    /// HTTPリクエストのボディを文字列として取得する
    /// UTF-8 でデコードする
    #[inline]
    pub async fn body_string(&mut self) -> Result<String, HttpError> {
        let body = self.body().await?;
        // Vec<u8> を String に変換
        String::from_utf8(body).map_err(|e| HttpError::InternalServerError(e.to_string()))
    }

    /// HTTPリクエストのボディを JSON として取得する
    /// serde_json::Value に変換する
    #[inline]
    pub async fn body_json(&mut self) -> Result<serde_json::Value, HttpError> {
        let body = self.body_string().await?;
        // JSON 文字列を serde_json::Value に変換
        serde_json::from_str(&body).map_err(|e| HttpError::InternalServerError(e.to_string()))
    }

    /// HTTPリクエストのボディをformデータとして取得する
    #[inline]
    pub async fn body_form(&mut self) -> Result<Vec<(String, String)>, HttpError> {
        let body = self.body_string().await?;
        // フォームデータを Vec<(String, String)> に変換
        let form_data: Vec<(String, String)> = body.split('&')
            .filter_map(|s| {
                let mut iter = s.splitn(2, '=');
                if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                    Some((key.to_string(), value.to_string()))
                } else {
                    None
                }
            })
            .collect();
        Ok(form_data)
    }

    /// HTTPリクエストのボディをストリームとして取得する
    /// BufReader を使用して、非同期に読み込む
    #[inline]
    pub async fn body_stream(&mut self) -> &mut BufReader<OwnedReadHalf> {
        self.connection.reader()    
    }

    #[inline]
    pub async fn body_de_struct<T: DeserializeOwned>(&mut self) -> Result<T, HttpError> {
        let body = self.body_string().await?;
        serde_json::from_str(&body).map_err(|e| HttpError::InternalServerError(e.to_string()))
    }
}
