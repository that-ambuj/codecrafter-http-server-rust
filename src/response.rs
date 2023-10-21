const OK_RESP: &str = "HTTP/1.1 200 OK\r\n";
const NOT_FOUND_RESP: &str = "HTTP/1.1 404 NOT FOUND\r\n";

#[derive(Default, Debug)]
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
    pub fn new_ok() -> Self {
        Response::Ok {
            content_type: ContentType::TextPlain,
            content_length: 0,
            body: String::new(),
        }
    }

    pub fn new_not_found() -> Self {
        Response::NotFound
    }

    pub fn set_body(self, body: &str) -> Self {
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

    pub fn build(&self) -> String {
        match self {
            Response::Ok {
                content_type,
                content_length,
                body,
            } => format!(
                "{OK_RESP}Content-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                content_type.to_string(),
                content_length,
                body
            ),
            Response::NotFound => format!("{NOT_FOUND_RESP}\r\n"),
        }
    }
}
