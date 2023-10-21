use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0, multispace1};
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::IResult;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

use crate::response::Response;

pub async fn parse_request(input: BufReader<&mut TcpStream>) -> Response {
    let mut lines = input.lines();

    if let Ok(Some(path_header)) = lines.next_line().await {
        match parse_path(&path_header) {
            Ok((_, path)) => match path {
                "/user-agent" => loop {
                    if let Ok(Some(line)) = lines.next_line().await {
                        if let Ok((_, ("User-Agent", v))) = parse_header_value(&line) {
                            return Response::new_ok().set_body(v);
                        }
                    }
                },
                res if res.starts_with("/echo") => {
                    Response::new_ok().set_body(remove_echo_prefix(res).unwrap().1)
                }
                "/" => Response::new_ok(),
                _ => Response::new_not_found(),
            },
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
