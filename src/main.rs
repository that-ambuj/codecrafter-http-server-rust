use nom::character::complete::alphanumeric1;
use nom::IResult;
use nom::{bytes::complete::tag, sequence::delimited};
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

const OK_RESP: &'static str = "HTTP/1.1 200 OK\r\n";
const NOT_FOUND_RESP: &'static str = "HTTP/1.1 404 NOT FOUND\r\n";

fn handle_stream(mut stream: TcpStream) -> std::io::Result<()> {
    let reader = BufReader::new(&mut stream);

    let mut lines = reader.lines();

    if let Some(first_line) = lines.next() {
        match first_line.as_deref() {
            Ok("GET / HTTP/1.1") => Response::new_ok().write_tcp(&mut stream)?,
            Ok(other) => {
                if let Ok((_, param)) = parse_echo_header(other) {
                    Response::new_ok().set_body(param).write_tcp(&mut stream)?
                } else {
                    Response::new_not_found().write_tcp(&mut stream)?
                }
            }
            _ => Response::new_not_found().write_tcp(&mut stream)?,
        }
    }

    Ok(())
}

fn parse_echo_header(input: &str) -> IResult<&str, &str> {
    delimited(tag("GET /echo/"), alphanumeric1, tag(" HTTP/1.1"))(input)
}

#[derive(Default)]
pub enum ContentType {
    #[default]
    TextPlain,
}

impl ToString for ContentType {
    fn to_string(&self) -> String {
        match self {
            ContentType::TextPlain => "text/plain".into(),
        }
    }
}

pub enum Response {
    Ok {
        content_type: ContentType,
        content_length: usize,
        body: String,
    },
    NotFound,
}

impl Default for Response {
    fn default() -> Self {
        Response::Ok {
            content_type: Default::default(),
            content_length: 0,
            body: String::new(),
        }
    }
}

impl Response {
    fn new_ok() -> Self {
        Response::Ok {
            content_type: ContentType::TextPlain,
            content_length: 0,
            body: String::new(),
        }
    }

    fn new_not_found() -> Self {
        Response::NotFound
    }

    fn write_tcp(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(&self.build().into_bytes())
    }

    fn set_body(self, body: &str) -> Self {
        match self {
            Response::Ok { content_type, .. } => Response::Ok {
                content_type,
                content_length: body.len(),
                body: body.to_string(),
            },
            // noop when setting body for not found type
            Response::NotFound => Response::NotFound,
        }
    }

    fn build(&self) -> String {
        match self {
            Response::Ok {
                content_type,
                content_length,
                body,
            } => format!(
                "{OK_RESP}Content-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                content_type.to_string(),
                content_length,
                body
            ),
            Response::NotFound => format!("{NOT_FOUND_RESP}\r\n"),
        }
    }
}
