use anyhow::Result;
use nom::multi::many1;
use std::path::PathBuf;

use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, line_ending, multispace0, multispace1, not_line_ending};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::IResult;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

use crate::response::{Code, ContentType, Response};

pub async fn parse_request(
    input: BufReader<&mut TcpStream>,
    file_dir: PathBuf,
) -> Result<Response> {
    let mut lines = input.lines();

    if let Ok(Some(path_header)) = lines.next_line().await {
        match (parse_path_get(&path_header), parse_path_post(&path_header)) {
            // For GET requests
            (Ok((_, path)), _) => match path {
                "/user-agent" => loop {
                    if let Ok(Some(line)) = lines.next_line().await {
                        if let Ok((_, ("User-Agent", v))) = parse_header_value(&line) {
                            return Ok(Response::new_ok().set_body(v.as_bytes()));
                        }
                    }
                },
                res if res.starts_with("/files") => {
                    let file_name = remove_files_prefix(res).unwrap().1;
                    let file_path = file_dir.join(file_name);

                    if let Ok(false) = file_path.try_exists() {
                        return Ok(Response::new_not_found());
                    }

                    let contents = fs::read(file_path).await?;

                    Ok(Response::new_ok()
                        .set_content_type(ContentType::Binary)
                        .set_body(&contents))
                }
                res if res.starts_with("/echo") => {
                    Ok(Response::new_ok().set_body(remove_echo_prefix(res).unwrap().1.as_bytes()))
                }
                "/" => Ok(Response::new_ok()),
                _ => Ok(Response::new_not_found()),
            },
            // for POST requests
            (_, Ok((_, path))) => match path {
                res if res.starts_with("/files") => {
                    let file_name = remove_files_prefix(res).unwrap().1;
                    let file_path = file_dir.join(file_name);

                    let contents = parse_post_body(res).unwrap().1;

                    fs::write(file_path, contents).await.unwrap();

                    return Ok(Response::new_ok()
                        .set_code(Code::Created)
                        .set_content_type(ContentType::Binary)
                        .set_body(contents.as_bytes()));
                }
                _ => Ok(Response::new_not_found()),
            },
            _ => Ok(Response::new_not_found()),
        }
    } else {
        Ok(Response::new_not_found())
    }
}

fn remove_echo_prefix(input: &str) -> IResult<&str, &str> {
    preceded(tag("/echo/"), is_not(" "))(input)
}

fn remove_files_prefix(input: &str) -> IResult<&str, &str> {
    preceded(tag("/files/"), is_not(" "))(input)
}

fn parse_path_get(input: &str) -> IResult<&str, &str> {
    delimited(
        pair(tag("GET"), multispace1),
        is_not(" "),
        pair(multispace1, tag("HTTP/1.1")),
    )(input)
}

fn parse_path_post(input: &str) -> IResult<&str, &str> {
    delimited(
        pair(tag("POST"), multispace1),
        is_not(" "),
        pair(multispace1, tag("HTTP/1.1")),
    )(input)
}

fn parse_post_body(input: &str) -> IResult<&str, &str> {
    separated_pair(
        pair(parse_path_post, many1(parse_header_value)),
        line_ending,
        not_line_ending,
    )(input)
    .map(|(rest, (_, last))| (rest, last))
}

fn parse_header_value(input: &str) -> IResult<&str, (&str, &str)> {
    terminated(
        separated_pair(is_not(":"), pair(char(':'), multispace0), not_line_ending),
        line_ending,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path_get("GET /hello/world HTTP/1.1"),
            Ok(("", "/hello/world"))
        );
    }

    #[test]
    fn test_parse_path_post() {
        assert_eq!(
            parse_path_post("POST /hello/world HTTP/1.1"),
            Ok(("", "/hello/world"))
        );
    }

    #[test]
    fn test_parse_header_value() {
        assert_eq!(
            parse_header_value("Content-Type: text/plain\r\n"),
            Ok(("", ("Content-Type", "text/plain")))
        );

        assert_eq!(
            parse_header_value("User-Agent: curl/7.64.1\r\n"),
            Ok(("", ("User-Agent", "curl/7.64.1")))
        )
    }

    #[test]
    fn test_parse_post_body() {
        assert_eq!(
            parse_post_body(
                "POST /files/skibiddy HTTP/1.1\r\nContent-Length: 103\r\n\r\nHumpty Dumpty\r\n",
            ),
            Ok(("\r\n", "Humpty Dumpty"))
        )
    }
}
