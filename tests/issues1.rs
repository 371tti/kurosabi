// https://github.com/371tti/kurosabi/issues/1

use kurosabi::Kurosabi;




// 371tti.netでも使ったバグ
// このコードで/tool/speed_runnerにアクセスすると、404が返ってくる
// 修正済み
#[test]
fn main() {
    env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("debug")).unwrap_or_else(|_| ());
    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/", |mut c| async move {
        c.res.text("../data/pages/index/index.html");
        c
    });

    kurosabi.get("/terms", |mut c| async move {
        c.res.text("../data/pages/index/terms/index.html");
        c
    });

    kurosabi.get("/license", |mut c| async move {
        c.res.text("../data/pages/index/license/index.html");
        c
    });

    kurosabi.get("/tools", |mut c| async move {
        c.res.text("../data/pages/index/tools/index.html");
        c
    });

    kurosabi.get("/tool/clock", |mut c| async move {
        c.res.text("../data/pages/index/tools/clock.html");
        c
    });

    kurosabi.get("/tool/string_converter", |mut c| async move {
        c.res.text("../data/pages/index/tools/string_converter.html");
        c
    });

    kurosabi.get("/game/speed_runner", |mut c| async move {
        c.res.text("../data/pages/index/tools/games/speed_runner.html");
        c
    });

    kurosabi.get("/login", |mut c| async move {
        c.res.text("../data/pages/index/login/index.html");
        c
    });

    kurosabi.get("/release", |mut c| async move {
        c.res.text("../data/pages/index/release/index.html");
        c
    });

    kurosabi.get("/index.html", |mut c| async move {
        c.res.text("../data/pages/index/index.html");
        c
    });

    kurosabi.get("/index", |mut c| async move {
        c.res.text("../data/pages/index/index.html");
        c
    });

    kurosabi.get("/menue.js", |mut c| async move {
        c.res.text("../data/pages/index/menue.js");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/style.css", |mut c| async move {
        c.res.text("../data/pages/index/style.css");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/backwapper.js", |mut c| async move {
        c.res.text("../data/pages/index/backwapper.js");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/backwapper.css", |mut c| async move {
        c.res.text("../data/pages/index/backwapper.css");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/box-load-anime.js", |mut c| async move {
        c.res.text("../data/pages/index/box-load-anime.js");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/modern-border.css", |mut c| async move {
        c.res.text("../data/pages/index/modern-border.css");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/modern-border.js", |mut c| async move {
        c.res.text("../data/pages/index/modern-border.js");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/tag.css", |mut c| async move {
        c.res.text("../data/pages/index/tag.css");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/copyable.js", |mut c| async move {
        c.res.text("../data/pages/index/copyable.js");
        c
    });

    kurosabi.get("/load-screen.js", |mut c| async move {
        c.res.text("../data/pages/index/load-screen.js");
        c
    });

    kurosabi.get("/371tti_icon.png", |mut c| async move {
        c.res.text("../data/pages/index/371tti_icon.png");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/banner.png", |mut c| async move {
        c.res.text("../data/pages/index/banner.png");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.get("/favicon.ico", |mut c| async move {
        c.res.text("../data/pages/index/favicon.ico");
        c
    });

    kurosabi.get("/robots.txt", |mut c| async move {
        c.res.text("../data/pages/index/robots.txt");
        c.res.header.set("Access-Control-Allow-Origin", "*");
        c
    });

    kurosabi.not_found_handler(|mut c| async move {
        c.res.code = 404;
        c
    });

    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .accept_threads(1)
        .port(8080)
        .thread(8)
        .queue_size(1000)
        .build();

    server.run();
}

// router の機能確認
#[test]
fn test_router() {
    env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("debug")).unwrap_or_else(|_| ());
    let mut kurosabi = Kurosabi::new();

    kurosabi.get("/", |mut c| async move {
        c.res.text("Hello, World!");
        c
    });

    kurosabi.get("/test", |mut c| async move {
        c.res.text("This is a test route.");
        c
    });

    kurosabi.get("/t/hello", |mut c| async move {
        c.res.text("Hello, Kurosabi!");
        c
    });

    kurosabi.get("/t/:name", |mut c| async move {
        let name = c.req.path.get_field("name").unwrap_or("Guest".to_string());
        c.res.text(format!("Hello, {}!", name).as_str());
        c
    });

    kurosabi.get("/wildcard/*", |mut c| async move {
        let path = c.req.path.get_field("*").unwrap_or("No path provided".to_string());
        c.res.text(format!("You accessed: {}", path).as_str());
        c
    });

    kurosabi.get("/*", |mut c| async move {
        c.res.text("This is a catch-all route.");
        c
    });

    kurosabi.not_found_handler(|mut c| async move {
        c.res.text("Custom 404 Not Found");
        c.res.code = 404;
        c
    });

    let mut server = kurosabi.server()
        .host([0, 0, 0, 0])
        .port(8081)
        .thread(4)
        .build();

    server.run();
}