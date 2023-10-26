use std::fs::DirBuilder;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use clap::Parser;
use request::process_request;

mod parser;
mod request;
mod response;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let Args { directory } = Args::parse();

    if let Ok(false) = directory.try_exists() {
        DirBuilder::new().create(directory.clone()).unwrap();
    }

    let dir = Arc::new(directory);
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let dir = dir.clone();
        thread::spawn(move || handle_stream(stream, dir));
    }

    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    /// Directory to serve files from
    #[arg(short, long, default_value = "static")]
    directory: PathBuf,
}

fn handle_stream(mut stream: TcpStream, dir: Arc<PathBuf>) {
    stream.set_ttl(5).unwrap();
    let response = process_request(&mut stream, dir).unwrap();

    stream.write_all(response.build().as_bytes()).unwrap();
}
