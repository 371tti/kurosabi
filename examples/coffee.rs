use std::io::Result;

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_util::compat::Compat;

use serde::{Deserialize, Serialize};

use kurosabi::{
    connection::Connection, http::method::HttpMethod, router::DefaultContext, server::tokio::KurosabiTokioServerBuilder,
};

const HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Coffee Order</title>
</head>
<body>
    <h1>Welcome to the Kurosabi Cafe</h1>
    <p>Use the /coffee endpoint to place your order.</p>
    <form id="orderForm">
        <label>
            Milk:
            <input type="checkbox" name="milk" id="milk">
        </label>
        <br>
        <label>Sugar:
            <select name="sugar" id="sugar">
                <option value="None">None</option>
                <option value="One">One</option>
                <option value="Two">Two</option>
                <option value="Three">Three</option>
            </select>
        </label>
        <br>
        <button type="submit">Order</button>
    </form>
    <pre id="result"></pre>
    <script>
        document.getElementById('orderForm').addEventListener('submit', async function(e) {
            e.preventDefault();
            const milk = document.getElementById('milk').checked;
            const sugar = document.getElementById('sugar').value;
            const order = { milk, sugar };
            const res = await fetch('/coffee', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(order)
            });
            const text = await res.text();
            document.getElementById('result').textContent = text;
        });
    </script>
</body>
</html>"#;

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    let server = KurosabiTokioServerBuilder::default().router_and_build(
        |mut conn: Connection<DefaultContext, Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>>| async move {
            let method = conn.req.method();

            match method {
                HttpMethod::GET => match conn.path_segs().as_ref() {
                    // GET /
                    [""] => conn.html_body(HTML),

                    _ => conn.set_status_code(404u16).no_body(),
                },
                HttpMethod::POST => match conn.path_segs().as_ref() {
                    ["coffee"] => {
                        let order = match conn.req.read_json_de::<CoffeeOrder>().await {
                            Ok(o) => o,
                            Err(_) => {
                                let conn = conn.set_status_code(400u16);
                                return match conn.json_body_serialized(&Coffee::Error) {
                                    Ok(conn) => conn,
                                    Err(p) => p.connection.set_status_code(500u16).text_body(""),
                                };
                            },
                        };

                        let coffee = match (order.milk, order.sugar) {
                            (false, Sugar::None) => Coffee::Black,
                            (true, Sugar::None) => Coffee::WithMilk,
                            (false, Sugar::One) => Coffee::WithSugar(1),
                            (false, Sugar::Two) => Coffee::WithSugar(2),
                            (false, Sugar::Three) => Coffee::WithSugar(3),
                            (true, Sugar::One) => Coffee::WithMilkAndSugar(1),
                            (true, Sugar::Two) => Coffee::WithMilkAndSugar(2),
                            (true, Sugar::Three) => Coffee::WithMilkAndSugar(3),
                        };

                        match conn.json_body_serialized(&coffee) {
                            Ok(conn) => conn,
                            Err(p) => p.connection.set_status_code(500u16).text_body(""),
                        }
                    },
                    _ => conn.set_status_code(404u16).no_body(),
                },
                _ => conn.set_status_code(405u16).no_body(),
            }
        },
    );
    server.run().await
}

#[derive(Deserialize)]
struct CoffeeOrder {
    milk: bool,
    sugar: Sugar,
}

#[derive(Deserialize)]
enum Sugar {
    None,
    One,
    Two,
    Three,
}

#[derive(Serialize)]
enum Coffee {
    Black,
    WithMilk,
    WithSugar(u8),
    WithMilkAndSugar(u8),
    Error,
}
