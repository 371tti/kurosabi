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

    /// ヘッダを取得する
    /// 任意のキーに対応するヘッダを線形探索します
    pub fn get(&self, key: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.to_ascii_uppercase() == key).map(|(_, v)| v.as_str())
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
}