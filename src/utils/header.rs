/// HTTPリクエストのヘッダ
/// 現行のHTTP/1.1の仕様に準拠
pub struct Header {
    /// ヘッダのキーと値のペア
    /// リニアサーチの方が早い
    pub headers: Vec<(String, String)>,
}

impl Header {
    pub fn new() -> Header {
        Header {
            headers: Vec::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.headers.push((key.to_string(), value.to_string()));
    }

    pub fn del(&mut self, key: &str) {
        self.headers.retain(|(k, _)| k.to_ascii_uppercase() != key);
    }

    pub fn dels(&mut self, key: &str) {
        self.headers.retain(|(k, _)| k.to_ascii_uppercase() != key);
    }

    /// ヘッダを取得する
    /// 任意のキーに対応するヘッダを線形探索します
    pub fn get(&self, key: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.to_ascii_uppercase() == key).map(|(_, v)| v.as_str())
    }

    pub fn gets(&self, key: &str) -> Vec<&str> {
        self.headers.iter().filter(|(k, _)| k.to_ascii_uppercase() == key).map(|(_, v)| v.as_str()).collect()
    }

    pub fn index_get(&self, index: usize) -> Option<(&str, &str)> {
        self.headers.get(index).map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn index_del(&mut self, index: usize) {
        self.headers.remove(index);
    }

    pub fn index(&self, key: &str) -> Option<usize> {
        self.headers.iter().position(|(k, _)| k.to_ascii_uppercase() == key)
    }

    pub fn indexs(&self, key: &str) -> Vec<usize> {
        self.headers.iter().enumerate().filter(|(_, (k, _))| k.to_ascii_uppercase() == key).map(|(i, _)| i).collect()
    }

    /// head: host を取得する
    pub fn get_host(&self) -> Option<&str> {
        self.get("HOST")
    }

    /// head: user-agent を取得する
    /// ユーザーが使用しているエージェントを取得する
    pub fn get_user_agent(&self) -> Option<&str> {
        self.get("USER-AGENT")
    }

    /// head: accept を取得する
    pub fn get_accept(&self) -> Option<&str> {
        self.get("ACCEPT")
    }

    /// head: accept-language を取得する
    pub fn get_accept_language(&self) -> Option<&str> {
        self.get("ACCEPT-LANGUAGE")
    }

    /// head: accept-encoding を取得する
    pub fn get_accept_encoding(&self) -> Option<&str> {
        self.get("ACCEPT-ENCODING")
    }

    /// head: accept-charset を取得する
    pub fn get_connection(&self) -> Option<&str> {
        self.get("CONNECTION")
    }

    /// head: get_referer を取得する
    /// referer はリクエスト元のURLを示す
    /// ^^^^^^^ これは公式なタイポです
    pub fn get_referer(&self) -> Option<&str> {
        self.get("REFERER")
    }

    /// head: get_cookie を取得する
    pub fn get_cookie(&self) -> Option<&str> {
        self.get("COOKIE")
    }

    /// head: get_content_length を取得する
    /// リクエストボディの長さを取得する
    pub fn get_content_length(&self) -> Option<&str> {
        self.get("CONTENT-LENGTH")
    }

    /// head: get_content_type を取得する
    pub fn get_content_type(&self) -> Option<&str> {
        self.get("CONTENT-TYPE")
    }

    /// head: get_authorization を取得する
    pub fn get_authorization(&self) -> Option<&str> {
        self.get("AUTHORIZATION")
    }
}

/// HTTPヘッダのenum
/// 現行のHTTP/1.1の仕様に準拠
pub enum Method {
    /// GETメソッド
    /// cash_able: yes
    GET,

    /// POSTメソッド
    /// cash_able: conditional
    POST,

    /// HEADメソッド
    /// cash_able: yes
    HEAD,

    /// PUTメソッド
    /// cash_able: no
    PUT,

    /// DELETEメソッド
    /// cash_able: no
    DELETE,

    /// OPTIONSメソッド
    /// cash_able: no
    OPTIONS,

    /// TRACEメソッド
    /// cash_able: no
    TRACE,

    /// CONNECTメソッド
    /// cash_able: no
    CONNECT,

    /// PATCHメソッド
    /// cash_able: conditional
    PATCH,

    /// カスタム
    UNKNOWN(String),
}

impl Method {
    /// 文字列からMethodを取得する
    pub fn from_str(method: &str) -> Option<Method> {
        match method {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "HEAD" => Some(Method::HEAD),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "OPTIONS" => Some(Method::OPTIONS),
            "TRACE" => Some(Method::TRACE),
            "CONNECT" => Some(Method::CONNECT),
            "PATCH" => Some(Method::PATCH),
            method => Some(Method::UNKNOWN(method.to_string())),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::HEAD => "HEAD",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
            Method::CONNECT => "CONNECT",
            Method::PATCH => "PATCH",
            Method::UNKNOWN(method) => method,
        }
    }
}