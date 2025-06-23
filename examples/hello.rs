use kurosabi::Kurosabi;

fn main() {
    // ログの初期化
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/", |mut c| async move {
        let html = include_str!("kurosabi.html");
        c.res.html(html);
        c
    });

    kurosabi.get("/kurosabi.css", |mut c| async move {
        let css = include_str!("kurosabi.css");
        c.res.css(css);
        c
    });

    kurosabi.server().build().run();
}