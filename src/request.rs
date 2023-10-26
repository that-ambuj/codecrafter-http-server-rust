use anyhow::Result;
use std::{collections::HashMap, fs, net::TcpStream, path::PathBuf, sync::Arc};

use crate::{
    parser::parse_request,
    response::{Code, ContentType, Response},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Request {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
}

pub fn process_request(stream: &mut TcpStream, file_dir: Arc<PathBuf>) -> Result<Response> {
    let mut buf = [0; 4096];
    let n = stream.peek(&mut buf)?;

    if n == 0 {
        // TODO: Send a 500 server error
        return Ok(Response::new_not_found());
    }

    let raw_req = String::from_utf8_lossy(&buf[..n]);
    let (_, req) = parse_request(&raw_req).unwrap();

    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/user-agent") => {
            let agent = req.headers.get("User-Agent").unwrap();
            Ok(Response::new_ok().set_body(agent.as_bytes()))
        }
        ("GET", "/") => Ok(Response::new_ok()),
        ("GET", path) => {
            if path.starts_with("/echo") {
                let word = path.trim_start_matches("/echo/");
                return Ok(Response::new_ok().set_body(word.as_bytes()));
            }

            if path.starts_with("/files") {
                let file_name = path.trim_start_matches("/files/");
                let file_path = file_dir.join(file_name);

                let contents = fs::read(file_path)?;

                return Ok(Response::new_ok()
                    .set_content_type(ContentType::Binary)
                    .set_body(&contents));
            }

            Ok(Response::new_not_found())
        }
        ("POST", path) if path.starts_with("/files") => {
            let file_name = path.trim_start_matches("/files/");
            let file_path = file_dir.join(file_name);

            let contents = req.body;

            fs::write(file_path, contents.trim())?;

            return Ok(Response::new_ok()
                .set_code(Code::Created)
                .set_body(contents.as_bytes()));
        }

        (_, _) => Ok(Response::new_not_found()),
    }
}
