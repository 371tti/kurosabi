
/// HTTPリクエストのヘッダ
/// 現行のHTTP/1.1の仕様に準拠
/// HTTP解析時にパースされる
pub struct Header {
    /// ヘッダのキーと値のペア
    /// リニアサーチの方が早い
    pub headers: Vec<(String, String)>,
}

/// 汎用な操作
impl Header {
    #[inline]
    pub fn new() -> Header {
        Header {
            headers: Vec::new(),
        }
    }

    /// ヘッダを追加する
    #[inline]
    pub fn set(&mut self, key: &str, value: &str) {
        let key = key.to_ascii_uppercase();
        self.headers.push((key.to_string(), value.to_string()));
    }

    /// ヘッダを削除する
    #[inline]
    pub fn del(&mut self, key: &str) {
        let key = key.to_ascii_uppercase();
        self.headers.retain(|(k, _)| k.to_ascii_uppercase() != key);
    }

    /// 1つ目に一致したヘッダーを置き換える または追加する
    /// delしてからsetするより効率的
    #[inline]
    pub fn replace(&mut self, key: &str, value: &str) {
        let key_upper = key.to_ascii_uppercase();
        for (k, v) in &mut self.headers {
            if k.to_ascii_uppercase() == key_upper {
                *v = value.to_string();
                return;
            }
        }
        self.headers.push((key_upper, value.to_string()));
    }

    #[inline]
    pub fn clear(&mut self) {
        self.headers.clear();
    }

    /// ヘッダを削除する
    /// 複数ある場合は、全て削除します
    #[inline]
    pub fn dels(&mut self, key: &str) {
        let key = key.to_ascii_uppercase();
        self.headers.retain(|(k, _)| k.to_ascii_uppercase() != key);
    }

    /// ヘッダを取得する
    /// 任意のキーに対応するヘッダを線形探索します
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        let key = key.to_ascii_uppercase();
        self.headers.iter().find(|(k, _)| k.to_ascii_uppercase() == key).map(|(_, v)| v.as_str())
    }

    /// ヘッダを取得する
    /// 任意のキーに対応するヘッダを線形探索します
    /// 複数の値を持つ場合は、Vecで返します
    #[inline]
    pub fn gets(&self, key: &str) -> Vec<&str> {
        let key = key.to_ascii_uppercase();
        self.headers.iter().filter(|(k, _)| k.to_ascii_uppercase() == key).map(|(_, v)| v.as_str()).collect()
    }

    /// indexのヘッダを取得する
    #[inline]
    pub fn index_get(&self, index: usize) -> Option<(&str, &str)> {
        self.headers.get(index).map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// indexのヘッダを削除する
    #[inline]
    pub fn index_del(&mut self, index: usize) {
        self.headers.remove(index);
    }

    /// ヘッダのインデックスを取得する
    /// 任意のキーに対応するヘッダを線形探索します
    #[inline]
    pub fn index(&self, key: &str) -> Option<usize> {
        let key = key.to_ascii_uppercase();
        self.headers.iter().position(|(k, _)| k.to_ascii_uppercase() == key)
    }

    /// ヘッダのインデックスを取得する
    /// 任意のキーに対応するヘッダを線形探索します
    /// 複数の値を持つ場合は、Vecで返します
    #[inline]
    pub fn indexs(&self, key: &str) -> Vec<usize> {
        let key = key.to_ascii_uppercase();
        self.headers.iter().enumerate().filter(|(_, (k, _))| k.to_ascii_uppercase() == key).map(|(i, _)| i).collect()
    }
}

/// HTTPヘッダの操作
impl Header {
    /// head: host を取得する
    #[inline]
    pub fn get_host(&self) -> Option<&str> {
        self.get("HOST")
    }

    /// head: user-agent を取得する
    /// ユーザーが使用しているエージェントを取得する
    #[inline]
    pub fn get_user_agent(&self) -> Option<&str> {
        self.get("USER-AGENT")
    }

    /// head: accept を取得する
    #[inline]
    pub fn get_accept(&self) -> Option<&str> {
        self.get("ACCEPT")
    }

    /// head: accept-language を取得する
    #[inline]
    pub fn get_accept_language(&self) -> Option<&str> {
        self.get("ACCEPT-LANGUAGE")
    }

    /// head: accept-encoding を取得する
    #[inline]
    pub fn get_accept_encoding(&self) -> Option<&str> {
        self.get("ACCEPT-ENCODING")
    }

    /// head: accept-charset を取得する
    #[inline]
    pub fn get_connection(&self) -> Option<&str> {
        self.get("CONNECTION")
    }

    /// head: get_referer を取得する
    /// referer はリクエスト元のURLを示す
    /// ^^^^^^^ これは公式なタイポです
    #[inline]
    pub fn get_referer(&self) -> Option<&str> {
        self.get("REFERER")
    }

    /// head: get_content_length を取得する
    /// リクエストボディの長さを取得する
    #[inline]
    pub fn get_content_length(&self) -> Option<&str> {
        self.get("CONTENT-LENGTH")
    }

    /// head: get_content_type を取得する
    #[inline]
    pub fn get_content_type(&self) -> Option<&str> {
        self.get("CONTENT-TYPE")
    }

    /// head: get_authorization を取得する
    #[inline]
    pub fn get_authorization(&self) -> Option<&str> {
        self.get("AUTHORIZATION")
    }
}

/// クッキーの操作
impl Header {
    /// head: get_cookie を取得する
    #[inline]
    pub fn get_cookie(&self, key: &str) -> Option<&str> {
        let cookie = self.gets("COOKIE");
        for c in cookie {
            for pair in c.split(';') {
                let pair = pair.trim();
                if let Some((k, v)) = pair.split_once('=') {
                    if k.trim() == key {
                        return Some(v.trim());
                    }
                }
            }
        }
        None
    }

    /// head: set_cookie をセットする
    #[inline]
    pub fn set_cookie(&mut self, key: &str, value: &str) {
        self.set("Set-Cookie", &format!("{}={}", key, value));
    }

    /// head: set_cookie をセットする
    /// 追加のオプション（例: Path, Domain, Expires, Secure, HttpOnly, SameSiteなど）に対応
    #[inline]
    pub fn set_cookie_options(&mut self, key: &str, value: &str, options: Option<&[(&str, &str)]>) {
        let mut cookie = format!("{}={}", key, value);
        if let Some(opts) = options {
            for (opt_key, opt_value) in opts {
                if opt_value.is_empty() {
                    // Secure, HttpOnly など値なしオプション
                    cookie.push_str(&format!("; {}", opt_key));
                } else {
                    cookie.push_str(&format!("; {}={}", opt_key, opt_value));
                }
            }
        }
        self.set("Set-Cookie", &cookie);
    }

    /// head: set_cookie_with_params をセットする
    #[inline]
    pub fn set_cookie_with_params(
        &mut self,
        key: &str,
        value: &str,
        secure: bool,
        http_only: bool,
        path: Option<&str>,
        same_site: Option<&str>,
        max_age: Option<i64>,
        domain: Option<&str>,
        expires: Option<&str>,
        priority: Option<&str>,
    ) {
        let mut cookie = format!("{}={}", key, value);
        if secure {
            cookie.push_str("; Secure");
        }
        if http_only {
            cookie.push_str("; HttpOnly");
        }
        if let Some(path) = path {
            cookie.push_str(&format!("; Path={}", path));
        }
        if let Some(domain) = domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }
        if let Some(expires) = expires {
            cookie.push_str(&format!("; Expires={}", expires));
        }
        if let Some(same_site) = same_site {
            cookie.push_str(&format!("; SameSite={}", same_site));
        }
        if let Some(max_age) = max_age {
            cookie.push_str(&format!("; Max-Age={}", max_age));
        }
        if let Some(priority) = priority {
            cookie.push_str(&format!("; Priority={}", priority));
        }
        self.set("Set-Cookie", &cookie);
    }

    /// head: del_cookie を削除する
    #[inline]
    pub fn del_cookie(&mut self, key: &str) {
        let indexs = self.indexs("Set-Cookie");
        for i in indexs {
            let (_, v) = self.index_get(i).expect("get header");
            if v.starts_with(&format!("{}=", key)) {
                self.index_del(i);
            }
        }
    }
}

impl Header {
    #[inline]
    pub fn get_accept_encoding_vec(&self) -> Option<Vec<&str>> {
        let accept_encoding = self.get("ACCEPT-ENCODING")?;
        let mut encodings: Vec<(&str, f32)> = accept_encoding
            .split(',')
            .map(|s| {
                let mut parts = s.trim().split(';');
                let encoding = parts.next().unwrap_or("").trim();
                let q_value = parts
                    .find(|p| p.trim().starts_with("q="))
                    .and_then(|q| q.trim().strip_prefix("q=").and_then(|v| v.parse::<f32>().ok()))
                    .unwrap_or(1.0); // デフォルトの品質値は1.0
                (encoding, q_value)
            })
            .collect();
    
        // 品質値で降順にソート
        encodings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
        // エンコーディング名のみを抽出して返す
        Some(encodings.into_iter().map(|(encoding, _)| encoding).collect())
    }
}
