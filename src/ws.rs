use websocket::sender::Sender;
use websocket::ws::sender::Sender as SenderTrait;
use websocket::Message;
use websocket::sync::Client;
use websocket::stream::sync::TcpStream;
use std::sync::mpsc;
use websocket::client::builder::ClientBuilder;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Command {
    pub angle: f64,
    pub throttle: f64,
    pub drive_mode: &'static str,
    pub recording: bool,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            angle: 0.0,
            throttle: 0.0,
            drive_mode: "user",
            recording: false,
        }
    }
}

fn ws_handler(mut ws: Client<TcpStream>, rx: mpsc::Receiver<Command>) -> anyhow::Result<()> {
    while let Ok(cmd) = rx.recv() {
        let cmd = serde_json::to_string(&cmd)?;
        let msg = Message::text(&cmd);
        let mut sender = Sender::new(true);
        let mut buf = Vec::new();
        sender.send_message(&mut buf, &msg)?;
        ws.writer_mut().write_all(&buf)?
    }

    Ok(())
}

pub fn connect_ws() -> anyhow::Result<mpsc::Sender<Command>> {
    let (tx, rx) = mpsc::channel();
    let ws = ClientBuilder::new("ws://192.168.81.154:8887/wsDrive")?.connect_insecure()?;

    std::thread::spawn(move || {
        if let Err(e) = ws_handler(ws, rx) {
            eprintln!("Error in WS thread: {}", e);
        }
    });

    Ok(tx)
}