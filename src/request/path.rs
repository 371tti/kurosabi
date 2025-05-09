pub struct Path {
    /// パスの文字列(完全)を保持
    pub path: String,
    segments: Segments,
    query: Query,
    fields: Vec<(String, String)>,
}

impl Path {
    #[inline]
    pub fn new(path: &str) -> Path {
        Path {
            path: path.to_string(),
            segments: Segments::new(),
            query: Query::new(),
            fields: Vec::new(),
        }
    }

    /// 生の全体パスを取得する
    /// 例: "/api/v1/user?id=123&name=John"
    #[inline]
    pub fn get_raw_path(&self) -> &str {
        &self.path
    }   

    /// パスを取得する(クエリパラメータを除去)
    #[inline]
    pub fn get_path(&mut self) -> String {
        if self.segments.segments.is_empty() {
            self.dec_segment();
        }
        self.segments.segments.join("/")
    }

    /// クエリパラメータを取得する
    #[inline]
    pub fn get_query(&mut self, key: &str) -> Option<String> {
        if self.query.query.is_empty() {
            self.dec_query();
        }
        self.query.query.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    /// セグメントのデコード
    #[inline]
    fn dec_segment(&mut self) {
        self.segments.segments = self.path.split('/')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
    }

    /// クエリパラメータのデコード
    #[inline]
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

    /// フィールドを取得する
    /// 事前定義済みのフィールド名を使用
    #[inline]
    pub fn get_field(&mut self, key: &str) -> Option<String> {
        self.fields.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    /// フィールドをセットする
    /// Routerのお仕事です
    #[inline]
    pub fn set_field(&mut self, key: &str, value: &str) {
        self.fields.push((key.to_string(), value.to_string()));
        
    }

    /// フィールドを削除する
    #[inline]
    pub fn remove_field(&mut self, key: &str) -> Option<String> {
        if let Some(pos) = self.fields.iter().position(|(k, _)| k == key) {
            let value = self.fields.remove(pos);
            Some(value.1)
        } else {
            None
        }
    }
}

/// セグメントを保持する構造体
pub struct Segments {
    pub segments: Vec<String>,
}

impl Segments {
    #[inline]
    pub fn new() -> Segments {
        Segments {
            segments: Vec::new(),
        }
    }
}

/// クエリパラメータを保持する構造体
pub struct Query {
    pub query: Vec<(String, String)>,
}

impl Query {
    #[inline]
    pub fn new() -> Query {
        Query {
            query: Vec::new(),
        }
    }
}