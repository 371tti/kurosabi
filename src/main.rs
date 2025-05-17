use std::{path::PathBuf, sync::Arc};

use kurosabi::{
    api::GETJsonAPI, html_format, kurosabi::Context, Kurosabi
};
use serde::Serialize;
use futures::stream::{StreamExt};

pub struct MyContext {
    pub name: String,
}

impl MyContext {
    pub fn new(name: String) -> Self {
        MyContext { name }
    }
}


/// json api(GET)の実装方法
/// kurosabiはjson apiをrustのstructで受け取り、送信できます (serdeによる)
/// 明確に構造化されたレスポンスを安全に受け取ったり送信したりできますよ
/// 
/// APIの実装用構造体
#[derive(Clone)]
pub struct MyAPI;

/// APIのレスポンス用構造体 要素
/// Serializeトレイトを実装している必要があります
#[derive(Serialize)]
pub struct ResJsonSchemaVersion {
    pub name: String,
    pub version: String,
}

/// APIのレスポンス用構造体 バリエーション
/// Serializeトレイトを実装している必要があります
/// `#[serde(untagged)]`を使うことで、異なる型のレスポンスを同じ列挙型で表現できます
/// これにより、APIのレスポンスが異なる形式を持つ場合でも、同じ型で処理できるようになります
#[derive(Serialize)]
#[serde(untagged)]
pub enum ResJsonSchema {
    Version(ResJsonSchemaVersion),
    Error(String),
}

/// APIの実装です
/// GETJsonAPIトレイトを実装することで、GETリクエストに対するハンドラを定義します
/// `Context<Arc<MyContext>>`は、リクエストのコンテキストを表します
/// `ResJsonSchema`は、レスポンスの型を表します
#[async_trait::async_trait]
impl GETJsonAPI<Context<Arc<MyContext>>, ResJsonSchema> for MyAPI {
    /// 新しいAPIのインスタンスを作成します
    fn new() -> Self {
        MyAPI
    }

    /// リクエストを処理するハンドラです
    /// `self`はAPIのインスタンスを表し、`c`はリクエストのコンテキストを表します
    async fn handler(
            self,
            c: &mut Context<Arc<MyContext>>,
        ) -> ResJsonSchema {
            // クエリパラメータからnameとversionを取得します
            let name = c.req.path.get_query("name").unwrap_or("Kurosabi".to_string());
            let version = c.req.path.get_query("version").unwrap_or("0.1".to_string());
            
            // レスポンスヘッダにConnectionとKeep-Aliveを設定します
            ResJsonSchema::Version(
                ResJsonSchemaVersion {
                    name: name,
                    version: version,
                }
            )
    }
}

#[tokio::main]
async fn main() {
    // ログの初期化
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();

    // Arc<Context>を作成します
    // Arcはスレッドセーフな参照カウント型で、複数のスレッドで共有できます
    // べつにArcじゃなくてもいいです(kurosabiはリクエスト毎にコンテキストをcloneします
    let arc_context = Arc::new(MyContext::new("Kurosabi".to_string()));

    // Kurosabiのインスタンスを作成します
    let mut kurosabi = Kurosabi::with_context(arc_context);

    // routerにさきほど定義したMyAPIを登録します
    // これにより、GETリクエストが来たときにMyAPIのhandlerが呼び出されます
    kurosabi.get_json_api("/jsonapi", MyAPI::new());


    kurosabi.get("/hello",  |mut c| async move {
        c.res.text("Hello, World!");
        let key = "session_id";
        let value = "123456";
        c.res.header.set_cookie(key, value);
        c.res.header.set("X-Custom-Header", "MyValue");
        c
    });

    // method GETで"/file"にアクセスしたときのハンドラを定義します
    // このハンドラは、README.mdファイルをレスポンスとして返します
    kurosabi.get("/file", |mut c| async move {
        // stream対応、 range byte対応です inline: true でブラウザで表示されます
        // inline: false でダウンロードされます
        let _ = c.res.file(&c.req, PathBuf::from("README.md"), true).await.unwrap();
        c
    });

    // method GETで"/hello/:name"にアクセスしたときのハンドラを定義します
    // このハンドラは、URLパスの:name部分を取得し、"Hello, {name}!"というテキストをレスポンスとして返します
    kurosabi.get("/hello/:name", |mut c| async move {
        // URLパスの:name部分を取得します(線形探索で実装されています)
        let name = c.req.path.get_field("name").unwrap_or("World".to_string());
        c.res.text(&format!("Hello, {}!", name));
        c
    });

    // method GETで"/field/:field/:value"にアクセスしたときのハンドラを定義します
    // このハンドラは、URLパスの:fieldと:value部分を取得し、"Field: {field}, Value: {value}"というテキストをレスポンスとして返します
    kurosabi.get("/field/:field/:value", |mut c| async move {
        let field = c.req.path.get_field("field").unwrap_or("unknown".to_string());
        let value = c.req.path.get_field("value").unwrap_or("unknown".to_string());
        c.res.text(&format!("Field: {}, Value: {}", field, value));
        c
    });

    // method GETで"/gurd/*"にアクセスしたときのハンドラを定義します
    // このハンドラは、URLパスの*部分を取得し、"Gurd: {path}"というテキストをレスポンスとして返します
    // *はワイルドカードで、任意の文字列を受け取ります
    kurosabi.get("/gurd/*", |mut c| async move {
        let path = c.req.path.get_field("*").unwrap_or("unknown".to_string());
        c.res.text(&format!("Gurd: {}", path));
        c
    });

    // method GETで"/json"にアクセスしたときのハンドラを定義します
    // このハンドラは、JSON形式のデータをレスポンスとして返します
    kurosabi.get("/json", |mut c| async move {
        let json_data = r#"{"name": "Kurosabi", "version": "0.1"}"#;
        c.res.json(json_data);
        c
    });

    // method POSTで"/gurd/:path"にアクセスしたときのハンドラを定義します
    // これはレスポンスデータをそのまま返します
    kurosabi.post("/submit", |mut c| async move {
        let body = match c.req.body_form().await {
            Ok(data) => data,
            Err(e) => {
                println!("Error receiving POST data: {}", e);
                c.res.set_status(400);
                return c;
            }
        };
        c.res.html(&format!("Received: {:?}", body));
        c
    });

    // method GETで"/submit"にアクセスしたときのハンドラを定義します    // このハンドラは、HTML形式のフォームをレスポンスとして返します
    kurosabi.get("/submit", |mut c| async move {
        c.res.html(r#"
        <form action="/submit" method="post">
            <input type="text" name="data" placeholder="Enter some data" />
            <button type="submit">Submit</button>
        </form>
        "#);
        c
    });

    kurosabi.get("/loopA", |mut c| async move {
        c.res.html("<a href=\"/loopB\">loopA</a>");
        c
    });

    kurosabi.get("/loopB", |mut c| async move {
        c.res.html("<a href=\"/loopA\">loopB</a>");
        c
    });

    // streamの実装
    // method GETで"/stream"にアクセスしたときのハンドラを定義します
    // このハンドラは、1秒ごとに現在の時刻を送信するストリームをレスポンスとして返します
    kurosabi.get("/stream", |mut c| async move {
        use bytes::Bytes;
        use tokio::time::{sleep, Duration};
        use futures::stream;
        use tokio_util::io::StreamReader;
        use std::pin::Pin;
        use tokio::io::AsyncRead;

        // Set Transfer-Encoding to "chunked"
        c.res.header.set("Transfer-Encoding", "chunked");

        // ストリームを生成: 1秒ごとに "count: n\n" を送信 (n=1..=10)
        let stream = stream::iter(1..=1000)
            .map(|n| async move {
            sleep(Duration::from_secs(1)).await;
            let now = std::time::SystemTime::now();
            let data = format!("current time (iteration {}): {:?}\n", n, now);
            let chunk_size = data.len();
            println!("Sending chunk: {}", data);
            format!("{:X}\r\n{}\r\n", chunk_size, data)
            })
            .buffered(1)
            .map(|s| Ok::<_, std::io::Error>(Bytes::from(s)))
            .chain(stream::once(async {
                Ok::<_, std::io::Error>(Bytes::from("0\r\n\r\n"))
            }));

        // StreamReaderでAsyncReadに変換
        let reader = StreamReader::new(stream);
        println!("StreamReader created");

        // Pin<Box<dyn AsyncRead + Send + Sync>>
        let boxed_stream: Pin<Box<dyn AsyncRead + Send + Sync>> = Box::pin(reader);

        c.res.stream(boxed_stream, 8192);
        println!("StreamReader created");
        c
    });

    // method GETでroot("/")にアクセスしたときのハンドラを定義します
    // このハンドラは、HTML形式のウェルカムメッセージをレスポンスとして返します
    kurosabi.get("/", |mut c| async move {
        c.res.html(r#"
        <h1>Welcome to Kurosabi!</h1>
        <p>Try the following routes:</p>
        <ul>
            <li><a href="/hello">/hello</a></li>
            <li><a href="/hello/kurosabi">/hello/kurosabi</a></li>
            <li><a href="/json">/json</a></li>
            <li><a href="/field/name/Kurosabi">/field/name/Kurosabi</a></li>
            <li><a href="/gurd/some/path">/gurd/some/path</a></li>
            <li><a href="/submit">/submit</a></li>
            <li><a href="/gurd/*">/gurd/*</a></li>
            <li><a href="/file">/file</a></li>
            <li><a href="/jsonapi">/jsonapi</a></li>
            <li><a href="/loopA">/loopA</a></li>
            <li><a href="/loopB">/loopB</a></li>
            <li><a href="/notfound">/notfound</a></li>
        </ul>
        "#);
        c
    });

    // 404 notfound のときのハンドラを定義します
    // このハンドラは、HTML形式の404エラーメッセージをレスポンスとして返します
    kurosabi.not_found_handler(|mut c| async move {
        let html = html_format!(
            "<h1>404 Not Found</h1>
            <p>The page you are looking for does not exist.</p>
            <p>debug: {{data}}</p>",
            data = c.req.header.get_user_agent().unwrap_or("unknown")
        );
        c.res.html(&html);
        c.res.set_status(404);
        c
    });

    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(8080)
        .thread(8)
        .thread_name("kurosabi-worker".to_string())
        .queue_size(128)
        .nodelay(false) // 細かいストリームの実装をする場合は、nodelayをfalseにすることをおすすめします
        .build();

    server.run().await;
}

