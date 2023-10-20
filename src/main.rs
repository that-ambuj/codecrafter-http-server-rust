use std::io::Write;
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    let resp = "HTTP/1.1 200 OK\r\n\r\n";

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                stream.write_all(resp.as_bytes()).unwrap();
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }

    Ok(())
}
