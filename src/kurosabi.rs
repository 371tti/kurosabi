use std::io::Read;

pub struct Kurosabi {
}

/// 初期化およびインスタンス操作を行うためのメソッドぐん
/// 
impl Kurosabi {
    pub fn new() -> Kurosabi {
        Kurosabi {
        }
    }
}

/// レジストリ操作メソッドたち
/// 
impl Kurosabi {
    /// httpのGETメソッドに対するルーティングを登録する
    pub fn get<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }

    /// httpのPOSTメソッドに対するルーティングを登録する
    pub fn post<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }

    /// httpのHEADメソッドに対するルーティングを登録する
    pub fn head<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }

    /// httpのPUTメソッドに対するルーティングを登録する
    pub fn put<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }

    /// httpのDELETEメソッドに対するルーティングを登録する
    pub fn delete<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }

    /// httpのOPTIONSメソッドに対するルーティングを登録する
    pub fn options<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }

    /// httpのTRACEメソッドに対するルーティングを登録する
    pub fn trace<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }

    /// httpのCONNECTメソッドに対するルーティングを登録する
    pub fn connect<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }

    /// httpのPATCHメソッドに対するルーティングを登録する
    pub fn patch<F>(&mut self, pattern: &str, handler: F) -> () 
    where F: AsyncFn(&mut Req) -> Res
    {

    }
}

pub struct Req {
    pub method: Method,
    pub path: Path,
    pub header: Header,
}

pub struct Path {
    /// パスの文字列(完全)を保持
    /// 遅延処理をする
    pub path: String,
    pub segments: Option<Vec<String>>,
    pub query: Option<Vec<(String, String)>>,

}

impl Path {
    pub fn new() -> Path {
        Path {
    
            path: String::new(),
            segments: None,
            query: None,
        }
    }

    pub fn dec_segment(&mut self) {
        self.segments = Some(self.path.split('/').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect());
    }

    pub fn dec_query(&mut self) {
        self.query = Some(self.path.split('?').nth(1).unwrap_or("").split('&').filter_map(|s| {
            let mut iter = s.split('=');
            if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                Some((key.to_string(), value.to_string()))
            } else {
                None
            }
        }).collect());
    }
}

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

pub struct Res {
    pub status: Status,
    pub header: Header,
    pub body: Body,
}

pub enum Body {
    Empty,
    Text(String),
    Stream(Box<dyn Read + Send + Sync>),
}

pub enum Status {
    /// 100 Continue
    Code100Continue,

    /// 200 OK
    Code200OK,

    /// 404 Not Found
    Code404NotFound,

    /// 500 Internal Server Error
    Code500InternalServerError,
}