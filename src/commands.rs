use crate::*;
use async_std::{
    os::unix::net::{UnixStream, UnixListener},
    io::{BufReader, BufRead},
    io::prelude::BufReadExt,
};
use serde::Deserialize;

/// Protocol is a json representation of AppCommand, followed by a newline
async fn handle_command_stream(stream: UnixStream, cmd_tx: Sender<AppCommand>) {
    let mut b = BufReader::new(stream);
    let mut buf = String::new();
    while let Ok(bytes_read) = b.read_line(&mut buf).await {
        // EOF
        if bytes_read == 0 { break; }
        // try to deserialize AppCommand from json
        if let Ok(cmd) = serde_json::from_str::<AppCommand>(&buf) {
            cmd_tx.send(cmd).await;
        }
        buf.clear(); 
    }
}

async fn prep_command_socket(path: &Path) {
    if path.exists().await {
        std::fs::remove_file(path)
            .expect("Could not remove old command socket"); 
    }
}

pub async fn start_command_socket(path: impl AsRef<Path>, cmd_tx: Sender<AppCommand>) {
    prep_command_socket(path.as_ref()).await;
    let s = UnixListener::bind(path).await.unwrap();
    loop {
        if let Ok((stream, _)) = s.accept().await {
            spawn(handle_command_stream(stream, cmd_tx.clone()));
        };
    }
    unreachable!()
}
