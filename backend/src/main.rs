use actix_web::{rt, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};

// {"event": "command", "content": "hello"}
// or
// {"event": "resize", "content": {"rows": 24, "cols": 80}}

#[derive(Serialize, Deserialize)]
struct Message {
    event: String,
    content: serde_json::Value,
}

fn handle_message(message: &str) -> String {
    let message: Message = serde_json::from_str(message).unwrap();
    match message.event.as_str() {
        "command" => {
            let content = message.content.as_str().unwrap();
            println!("command: {}", content);
            format!("command: {}", content)
        }
        "resize" => {
            let content = message.content.as_object().unwrap();
            let rows = content.get("rows").unwrap().as_u64().unwrap();
            let cols = content.get("cols").unwrap().as_u64().unwrap();
            println!("resize: rows: {}, cols: {}", rows, cols);
            format!("resize: rows: {}, cols: {}", rows, cols)
        }
        _ => {
            println!("unknown event: {}", message.event);
            format!("unknown event: {}", message.event)
        }
    }
}

async fn new_client(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    // process text message
                    let response = handle_message(&text);
                    session.text(response).await.unwrap();
                }

                Ok(AggregatedMessage::Binary(bin)) => {
                    // echo binary messages
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    // respond to PING frame with PONG frame
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(new_client)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
