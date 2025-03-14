use crate::utils::header::{Header, Method};

pub struct Req {
    pub method: Method,
    pub path: Path,
    pub header: Header,
}

impl Req {
    pub fn new() -> Req {
        Req {
            method: Method::GET,
            path: Path::new(),
            header: Header::new(),
        }
    }
}

pub struct Path {
    /// パスの文字列(完全)を保持
    /// 遅延処理をする
    pub path: String,
    pub segments: Segments,
    pub query: Query,
}

impl Path {
    pub fn new() -> Path {
        Path {
            path: String::new(),
            segments: Segments::new(),
            query: Query::new(),
        }
    }

    pub fn dec_segment(&mut self) {
        self.segments.segments = self.path.split('/')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
    }

    pub fn dec_query(&mut self) {
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