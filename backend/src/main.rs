use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;
use std::io::{Read, Write};
use tokio::net::TcpListener;
use tokio::sync::mpsc::unbounded_channel;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;
use serde::Deserialize;

// {"event": "command", "content": "hello"}
// or
// {"event": "resize", "content": {"rows": 24, "cols": 80}}

#[derive(Deserialize, Debug)]
#[serde(tag = "event", content = "content", rename_all = "lowercase")]
enum ClientMessage {
    Command(String),
    Resize { rows: u16, cols: u16 },
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Failed to bind");
    println!("WebSocket server running on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let websocket = accept_async(stream).await.expect("Failed to accept connection");
    let (ws_sender, mut ws_receiver) = websocket.split();

    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let cmd = CommandBuilder::new("bash");
    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().unwrap();
    let master_writer = pair.master.take_writer().unwrap();

    // PTY -> WebSocket
    let mut ws_sender_clone = ws_sender;
    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                    println!("Output: {}", output);
                    match ws_sender_clone.send(Message::Text(output)).await {
                        Ok(_) => println!("Sent output to WebSocket"),
                        Err(e) => eprintln!("Failed to send to WebSocket: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from PTY: {}", e);
                    break;
                }
            }
        }
    });

    // WebSocket -> PTY
    let (tx, rx) = unbounded_channel::<ClientMessage>();
    tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = ws_receiver.next().await {
            match serde_json::from_str::<ClientMessage>(&msg) {
                Ok(parsed_msg) => {
                    if tx.send(parsed_msg).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to deserialize message: {}", e);
                }
            }
        }
    });

    // Handle PTY input from WebSocket
    tokio::spawn(async move {
        handle_input_stream(rx, master_writer, pair.master).await;
    });

    child.wait().unwrap();
    println!("Bash exited");
}

async fn handle_input_stream(
    mut rx: tokio::sync::mpsc::UnboundedReceiver<ClientMessage>,
    mut writer: Box<dyn Write + Send>,
    master: Box<(dyn portable_pty::MasterPty + std::marker::Send + 'static)>,
) {
    while let Some(input) = rx.recv().await {
        match input {
            ClientMessage::Command(cmd) => {
                if writer.write_all(cmd.as_bytes()).is_err() {
                    eprintln!("Error writing to PTY");
                    break;
                }
            }
            ClientMessage::Resize { rows, cols } => {
                if master.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .is_err()
                {
                    eprintln!("Error resizing PTY");
                }
            }
        }
    }
}
