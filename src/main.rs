use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use std::str;

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind("127.0.0.1:8181").await.unwrap();

  loop {
    let (stream, _) = listener.accept().await.unwrap();
    tokio::spawn(async move {
      handle_connection(stream).await;
    });
    // handle_connection(stream).await;
  }    
}

async fn handle_connection (mut stream: TcpStream) {
  let mut buffer = [0;16];
  stream.read(&mut buffer).await.unwrap();
  let method = match str::from_utf8(&buffer[0..4]) {
    Ok(v) => v,
    Err(e) => {
      panic!("Invalid UTF-8 sequence {}", e)
    }
  };

  // let contents = "{\"balance\": 0.00}";
  let contents = match method {
    "GET " => {
      format!("{{\"balance\": {}}}", 0.0)
    },
    "POST" => {
      let input: String = buffer [6..16]
        .iter()
        .take_while(|x| **x != 32u8)
        .map(|x| *x as char)
        .collect();
      let balance_update = input.parse::<f32>().unwrap();
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
