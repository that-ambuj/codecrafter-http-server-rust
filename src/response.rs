const OK_RESP: &str = "HTTP/1.1 200 OK\r\n";
const NOT_FOUND_RESP: &str = "HTTP/1.1 404 NOT FOUND\r\n";

#[derive(Default, Debug)]
pub enum ContentType {
    #[default]
    TextPlain,
    Binary,
}

impl ToString for ContentType {
    fn to_string(&self) -> String {
        match self {
            ContentType::TextPlain => "text/plain".into(),
            ContentType::Binary => "application/octet-stream".into(),
        }
    }
}

pub enum Response {
    Ok {
        content_type: ContentType,
        content_length: usize,
        body: Vec<u8>,
    },
    NotFound,
}

impl Response {
    pub fn new_ok() -> Self {
        Response::Ok {
            content_type: ContentType::TextPlain,
            content_length: 0,
            body: Vec::new(),
        }
    }

    pub fn new_not_found() -> Self {
        Response::NotFound
    }

    pub fn set_content_type(self, content_type: ContentType) -> Self {
        match self {
            Response::Ok {
                content_length,
                body,
                // ignore existing content_type
                ..
            } => Response::Ok {
                content_type,
                content_length,
                body,
            },
            Response::NotFound => Response::NotFound,
        }
    }

    pub fn set_body(self, body: &[u8]) -> Self {
        match self {
            Response::Ok { content_type, .. } => Response::Ok {
                content_type,
                content_length: body.len(),
                body: body.to_owned(),
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
                String::from_utf8_lossy(body),
            ),
            Response::NotFound => format!("{NOT_FOUND_RESP}\r\n"),
        }
    }
}
