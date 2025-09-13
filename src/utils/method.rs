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
    #[inline]
    pub fn from_str(method: &str) -> Method {
        match method {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "HEAD" => Method::HEAD,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "OPTIONS" => Method::OPTIONS,
            "TRACE" => Method::TRACE,
            "CONNECT" => Method::CONNECT,
            "PATCH" => Method::PATCH,
            method => Method::UNKNOWN(method.to_string()),
        }
    }

    #[inline]
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
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}