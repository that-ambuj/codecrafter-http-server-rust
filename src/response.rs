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

#[derive(Default, Debug)]
pub enum Code {
    #[default]
    Ok,
    Created,
    NotFound,
}

impl ToString for Code {
    fn to_string(&self) -> String {
        match self {
            Code::Ok => "200 OK",
            Code::Created => "201 CREATED",
            Code::NotFound => "404 NOT FOUND",
        }
        .into()
    }
}

#[derive(Default)]
pub struct Response {
    code: Code,
    content_type: ContentType,
    body: Vec<u8>,
}

impl Response {
    pub fn new_ok() -> Self {
        Default::default()
    }

    pub fn new_not_found() -> Self {
        Response {
            code: Code::NotFound,
            ..Default::default()
        }
    }

    pub fn set_code(self, code: Code) -> Self {
        Response { code, ..self }
    }

    pub fn set_content_type(self, content_type: ContentType) -> Self {
        Response {
            content_type,
            ..self
        }
    }

    pub fn set_body(self, body: &[u8]) -> Self {
        Response {
            body: body.to_owned(),
            ..self
        }
    }

    pub fn build(&self) -> String {
        format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.code.to_string(),
            self.content_type.to_string(),
            self.body.len(),
            String::from_utf8_lossy(&self.body),
        )
    }
}
