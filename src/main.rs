use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

mod parser;
mod response;

use crate::parser::parse_request;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            handle_stream(stream).await;
        });
    }
}

async fn handle_stream(mut stream: TcpStream) {
    let reader = BufReader::new(&mut stream);

    let response = parse_request(reader).await;
    stream.write_all(response.build().as_bytes()).await.unwrap();
}
