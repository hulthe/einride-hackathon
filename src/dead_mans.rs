use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;
use crate::ws::Command;
use std::sync::mpsc::TryRecvError;


fn dms(ws: mpsc::Sender<Command>, rx: mpsc::Receiver<Command>) -> anyhow::Result<()> {
    
    let mut last_cmd = Command::default();
    loop {

        match rx.try_recv() {
            Ok(cmd) => {
                last_cmd = cmd;
                ws.send(cmd)?;
            }
            Err(TryRecvError::Empty) => {
                sleep(Duration::from_millis(10));
                if last_cmd.throttle > 0.0 {
                    last_cmd.throttle = (last_cmd.throttle - 0.01).max(0.0);
                }    
            }
            Err(_) => {
                eprintln!("DMS disconnected");
                return Ok(());
            }
        }
    }
}

pub fn start_dms(ws: mpsc::Sender<Command>) -> anyhow::Result<mpsc::Sender<Command>> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        if let Err(e) = dms(ws, rx) {
            eprintln!("Error in WS thread: {}", e);
        }
    });

    Ok(tx)
}