use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, multispace1};
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::IResult;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                handle_stream(stream).await.unwrap();
            });
        }
    }
}

const OK_RESP: &str = "HTTP/1.1 200 OK\r\n";
const NOT_FOUND_RESP: &str = "HTTP/1.1 404 NOT FOUND\r\n";

async fn handle_stream(mut stream: TcpStream) -> std::io::Result<()> {
    let reader = BufReader::new(&mut stream);

    let response = parse_request(reader).await;
    response.write_tcp(&mut stream).await;

    Ok(())
}

async fn parse_request(input: BufReader<&mut TcpStream>) -> Response {
    let mut lines = input.lines();

    let path_header = lines.next_line().await.unwrap().unwrap();

    if let Ok((_, path)) = parse_path(&path_header) {
        match path {
            "/user-agent" => loop {
                if let Ok(Some(line)) = lines.next_line().await {
                    if let Ok((_, ("User-Agent", v))) = parse_header_value(&line) {
                        break Response::new_ok().set_body(v);
                    } else {
                        continue;
                    }
                }
            },
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

    async fn write_tcp(&self, stream: &mut TcpStream) {
        while stream.write(&self.build().into_bytes()).await.is_ok() {}
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
