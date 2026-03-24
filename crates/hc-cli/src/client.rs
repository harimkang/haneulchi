use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

pub struct ControlClient {
    socket_path: PathBuf,
}

impl ControlClient {
    pub fn from_env() -> Result<Self, String> {
        let socket_path = match env::var("HC_CONTROL_SOCKET").ok().as_deref() {
            Some(path) => PathBuf::from(path),
            None => {
                let home = env::var("HOME").map_err(|error| error.to_string())?;
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("Haneulchi")
                    .join("run")
                    .join("control.sock")
            }
        };
        Ok(Self { socket_path })
    }

    pub fn get_json(&self, path: &str) -> Result<String, String> {
        self.request("GET", path, None)
    }

    pub fn post_json(&self, path: &str, body: Option<&str>) -> Result<String, String> {
        self.request("POST", path, body)
    }

    fn request(&self, method: &str, path: &str, body: Option<&str>) -> Result<String, String> {
        let mut stream = UnixStream::connect(&self.socket_path).map_err(|error| error.to_string())?;
        let body = body.unwrap_or("");
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| error.to_string())?;
        stream.flush().map_err(|error| error.to_string())?;
        stream.shutdown(std::net::Shutdown::Write).map_err(|error| error.to_string())?;
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|error| error.to_string())?;
        let (head, body) = response
            .split_once("\r\n\r\n")
            .ok_or_else(|| "invalid_http_response".to_string())?;
        let status = head
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|value| value.parse::<u16>().ok())
            .ok_or_else(|| "invalid_http_status".to_string())?;
        if status >= 400 {
            return Err(body.to_string());
        }
        Ok(body.to_string())
    }
}
