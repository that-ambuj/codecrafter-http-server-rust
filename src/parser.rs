use std::collections::HashMap;

use nom::branch::alt;

use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, crlf, multispace0, not_line_ending};
use nom::combinator::{opt, rest};
use nom::multi::separated_list0;
use nom::sequence::{pair, separated_pair};
use nom::IResult;

use crate::request::Request;

pub(crate) fn parse_request(input: &str) -> IResult<&str, Request> {
    let (input, (method, path)) = parse_method_path(input)?;
    let (input, _) = opt(crlf)(input)?;
    let (input, headers) = parse_header_list(input)?;
    let (input, _) = opt(crlf)(input)?;
    let (input, _) = opt(crlf)(input)?;
    let (input, body) = opt(rest)(input)?;

    let headers = headers
        .into_iter()
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .collect::<HashMap<_, _>>();

    Ok((
        input,
        Request {
            method: method.to_string(),
            headers,
            path: path.to_owned(),
            body: body.unwrap_or("").to_owned(),
        },
    ))
}

fn parse_method_path(input: &str) -> IResult<&str, (&str, &str)> {
    let (rest, (p, _)) = separated_pair(
        separated_pair(alt((tag("GET"), tag("POST"))), multispace0, is_not(" ")),
        multispace0,
        not_line_ending,
    )(input)?;

    Ok((rest, p))
}

fn parse_header(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(is_not(": "), pair(char(':'), char(' ')), not_line_ending)(input)
}

fn parse_header_list(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    separated_list0(crlf, parse_header)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path_post() {
        assert_eq!(
            parse_method_path("POST /hello/world HTTP/1.1\r\n"),
            Ok(("\r\n", ("POST", "/hello/world")))
        );
    }

    #[test]
    fn test_parse_request() {
        assert_eq!(
            parse_request("GET /hello/world HTTP/1.1\r\nContent-Length: 11\r\n\r\nHello World\r\n"),
            Ok((
                "",
                Request {
                    method: "GET".into(),
                    path: "/hello/world".to_string(),
                    headers: HashMap::from([("Content-Length".to_owned(), 11.to_string())]),
                    body: "Hello World\r\n".to_string()
                }
            ))
        );

        assert_eq!(
            parse_request("POST /files/world HTTP/1.1\r\nContent-Type: application/octet-stream\r\n\r\nHello Again!\r\n"),
            Ok((
                "",
                Request {
                    method: "POST".into(),
                    path: "/files/world".to_string(),
                    headers: HashMap::from([("Content-Type".to_owned(), "application/octet-stream".to_string())]),
                    body: "Hello Again!\r\n".to_string()
                }
            ))
        )
    }
}
