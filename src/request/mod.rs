use std::sync::Arc;

use tokio::{io::{AsyncBufReadExt, AsyncReadExt}, sync::Mutex};

use crate::{error::KurosabiError, server::TcpConnection, utils::header::{Header, Method}};

pub struct Req {
    pub method: Method,
    pub path: Path,
    pub header: Header,
    pub version: String,
    pub connection: TcpConnection,
}

impl Req {
    pub fn new(connection: TcpConnection) -> Req {
        Req {
            method: Method::UNKNOWN("until parse".to_string()),
            path: Path::new(""),
            header: Header::new(),
            version: String::new(),
            connection: connection,
        }
    }

    pub async fn parse_headers(&mut self) -> Result<(), KurosabiError> {
        let reader = self.connection.reader();
        let mut line_buf = String::with_capacity(1024);

        // Parse the request line first
        reader
            .read_line(&mut line_buf)
            .await
            .map_err(KurosabiError::IoError)?;
        let parts: Vec<&str> = line_buf.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err(KurosabiError::InvalidHttpHeader(line_buf));
        }

        let method = Method::from_str(parts[0]).unwrap();
        let path = Path::new(parts[1]);
        let version = parts[2].to_string();
        let mut header = Header::new();

        loop {
            line_buf.clear();
            reader
                .read_line(&mut line_buf)
                .await
                .map_err(KurosabiError::IoError)?;
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

    pub async fn body(&mut self) -> String {
        let mut buf = String::new();
        let reader = self.connection.reader();
        reader.read_to_string(&mut buf).await.unwrap();
        let _ = reader;
        buf
    }
}


pub struct Path {
    /// パスの文字列(完全)を保持
    /// 遅延処理をする
    pub path: String,
    segments: Segments,
    query: Query,
    fields: Vec<(String, String)>,
}

impl Path {
    pub fn new(path: &str) -> Path {
        Path {
            path: path.to_string(),
            segments: Segments::new(),
            query: Query::new(),
            fields: Vec::new(),
        }
    }

    pub fn get_raw_path(&self) -> &str {
        &self.path
    }   

    pub fn get_path(&mut self) -> String {
        self.dec_segment();
        self.segments.segments.join("/")
    }

    pub fn get_query(&mut self, key: &str) -> Option<String> {
        self.dec_query();
        self.query.query.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn dec_segment(&mut self) {
        self.segments.segments = self.path.split('/')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
    }

    fn dec_query(&mut self) {
        self.query.query = self.path.split('?')
            .nth(1)
            .unwrap_or("")
            .split('&')
            .filter_map(|s| {
                let mut iter = s.splitn(2, '=');
                if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                    Some((key.to_string(), value.to_string()))
                } else {
                    None
                }
            })
            .collect();
    }

    pub fn get_field(&mut self, key: &str) -> Option<String> {
        self.fields.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    pub fn set_field(&mut self, key: &str, value: &str) {
        self.fields.push((key.to_string(), value.to_string()));
        
    }

    pub fn remove_field(&mut self, key: &str) -> Option<String> {
        if let Some(pos) = self.fields.iter().position(|(k, _)| k == key) {
            let value = self.fields.remove(pos);
            Some(value.1)
        } else {
            None
        }
    }
}

pub struct Segments {
    pub segments: Vec<String>,
}

impl Segments {
    pub fn new() -> Segments {
        Segments {
            segments: Vec::new(),
        }
    }
}

pub struct Query {
    pub query: Vec<(String, String)>,
}

impl Query {
    pub fn new() -> Query {
        Query {
            query: Vec::new(),
        }
    }
}