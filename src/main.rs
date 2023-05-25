use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex, MutexGuard};

#[tokio::main]
async fn main() {
  let balance = Arc::new(Mutex::new(0.00f32));
  let listener = TcpListener::bind("127.0.0.1:8181").await.unwrap();

  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let balance = balance.clone();
    tokio::spawn(async move {
      handle_connection(stream, balance).await;
    });
    // handle_connection(stream).await;
  }    
}

async fn handle_connection (mut stream: TcpStream, balance: Arc<Mutex<f32>>) {
  let mut buffer = [0;16];
  stream.read(&mut buffer).await.unwrap();
  let method = match str::from_utf8(&buffer[0..4]) {
    Ok(v) => v,
    Err(e) => {
      panic!("Invalid UTF-8 sequence {}", e)
    }
  };
  let contents = match method {
    "GET " => {
      format!("{{\"balance\": {}}}", balance.lock().unwrap())
    },
    "POST" => {
      let input: String = buffer [6..16]
        .iter()
        .take_while(|x| **x != 32u8)
        .map(|x| *x as char)
        .collect();
      let balance_update = input.parse::<f32>().unwrap();
      let mut locked_balance: MutexGuard<f32> = balance.lock().unwrap();
      *locked_balance += balance_update;
      println!("balance: {}", balance_update);
      format!("{{\"balance\":{}}}", balance_update)
    },
    _ => {
      panic!("Invalid HTTP method")
    }
  };

  let response = format!(
    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", 
    contents.len(),
    contents);

  stream.write(response.as_bytes()).await.unwrap();
  stream.flush().await.unwrap();
}
