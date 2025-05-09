/// HTTPヘッダのenum
/// 現行のHTTP/1.1の仕様に準拠
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}