use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, multispace1};
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::IResult;
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

const OK_RESP: &str = "HTTP/1.1 200 OK\r\n";
const NOT_FOUND_RESP: &str = "HTTP/1.1 404 NOT FOUND\r\n";

fn handle_stream(mut stream: TcpStream) -> std::io::Result<()> {
    let reader = BufReader::new(&mut stream);

    let response = parse_request(reader);

    response.write_tcp(&mut stream)?;

    Ok(())
}

fn parse_request(input: BufReader<&mut TcpStream>) -> Response {
    let mut lines = input
        .lines()
        .filter_map(Result::ok)
        .filter(|l| !l.is_empty());

    let path_header = lines.next().unwrap();

    if let Ok((_, path)) = parse_path(&path_header) {
        match path {
            "/user-agent" => lines
                .find_map(|line| {
                    if let Ok((_, ("User-Agent", v))) = parse_header_value(&line) {
                        Some(v.to_string())
                    } else {
                        None
                    }
                })
                .map(|agent| Response::new_ok().set_body(&agent))
                .unwrap_or(Response::new_not_found()),
            "/" => Response::new_ok(),
            res if res.starts_with("/echo") => {
                Response::new_ok().set_body(remove_echo_prefix(res).unwrap().1)
            }
            _ => Response::new_not_found(),
        }
    } else {
        Response::new_not_found()
    }
}

fn remove_echo_prefix(input: &str) -> IResult<&str, &str> {
    preceded(tag("/echo/"), is_not(" "))(input)
}

fn parse_echo_header(input: &str) -> IResult<&str, &str> {
    parse_path(input).and_then(|(_, res)| remove_echo_prefix(res))
}

fn parse_path(input: &str) -> IResult<&str, &str> {
    delimited(
        pair(tag("GET"), multispace1),
        is_not(" "),
        pair(multispace1, tag("HTTP/1.1")),
    )(input)
}

fn parse_header_value(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(is_not(":"), pair(char(':'), multispace0), is_not(" "))(input)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_param() {
        assert_eq!(
            parse_echo_header("GET /echo/hello HTTP/1.1"),
            Ok(("", "hello"))
        );
        assert_eq!(
            parse_echo_header("GET /echo/217/delta HTTP/1.1"),
            Ok(("", "217/delta"))
        );
    }

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path("GET /hello/world HTTP/1.1"),
            Ok(("", "/hello/world"))
        );
    }

    #[test]
    fn test_parse_header_value() {
        assert_eq!(
            parse_header_value("Content-Type: text/plain"),
            Ok(("", ("Content-Type", "text/plain")))
        );

        assert_eq!(
            parse_header_value("User-Agent: curl/7.64.1"),
            Ok(("", ("User-Agent", "curl/7.64.1")))
        )
    }
}
