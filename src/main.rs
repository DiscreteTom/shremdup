mod manager;
mod model;
mod server;

use crate::{manager::manager_thread, server::server_thread};
use model::ServerMutex;
use std::{env::args, sync::Arc};
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() {
  let args: Vec<String> = args().collect();

  if args.len() != 2 {
    println!("Usage: {} <port>", args[0]);
    return;
  }

  // parse port from args
  let port: u16 = match args[1].parse() {
    Ok(port) => port,
    Err(_) => {
      println!("Invalid port: {}", args[1]);
      return;
    }
  };

  let (req_tx, req_rx) = mpsc::channel(1);
  let (res_tx, res_rx) = mpsc::channel(1);
  let mutex: ServerMutex = Arc::new(Mutex::new((req_tx, res_rx)));

  tokio::spawn(async move {
    println!("running on localhost:{}", port);
    server_thread(mutex, port).await;
  });

  manager_thread(req_rx, res_tx).await;
}
