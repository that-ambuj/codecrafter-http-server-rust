use std::path::PathBuf;

use tokio::fs::DirBuilder;

use clap::Parser;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

mod parser;
mod response;

use crate::parser::parse_request;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    let Args { directory } = Args::parse();

    if let Ok(false) = directory.try_exists() {
        DirBuilder::new().create(directory.clone()).await.unwrap();
    }

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_stream(stream, directory.clone()));
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Directory to serve files from
    #[arg(short, long, default_value = "static")]
    directory: PathBuf,
}

async fn handle_stream(mut stream: TcpStream, dir: PathBuf) {
    let reader = BufReader::new(&mut stream);

    let response = parse_request(reader, dir).await.unwrap();
    stream.write_all(response.build().as_bytes()).await.unwrap();
}
