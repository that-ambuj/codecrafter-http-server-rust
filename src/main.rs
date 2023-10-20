use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_stream(stream)?,
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }

    Ok(())
}

const OK_RESP: &'static str = "HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_RESP: &'static str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

fn handle_stream(mut stream: TcpStream) -> std::io::Result<()> {
    let reader = BufReader::new(&mut stream);

    let mut lines = reader.lines();

    if let Some(first_line) = lines.next() {
        match first_line.as_deref() {
            Ok("GET / HTTP/1.1") => stream.write_all(OK_RESP.as_bytes())?,
            _ => stream.write_all(NOT_FOUND_RESP.as_bytes())?,
        }
    }

    Ok(())
}
