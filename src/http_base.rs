use std::net::{TcpStream};
use std::io::{Bytes, BufRead, BufReader, Read, Write, BufWriter};
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::result::{Result as StdResult};

enum Method {
    GET,
    OPTIONS,
    HEAD,
    Other(String),
}
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            Method::GET => "GET",
            Method::OPTIONS => "OPTIONS",
            Method::HEAD => "HEAD",
            Method::Other(v) => v
        };

        write!(f, "{}", printable)
    }
}

struct Request {
    method: Method,
    version: String,
    path: String,

    headers: HashMap<String, String>,

    body: Bytes<TcpStream>,
}

type Result<T> = StdResult<T, RequestParseError>;

#[derive(Debug, Clone)]
struct RequestParseError;
impl RequestParseError {
    fn new() -> RequestParseError {
        return RequestParseError {}
    }
}
impl fmt::Display for RequestParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error parsing http request")
    }
}
impl error::Error for RequestParseError {
    fn description(&self) -> &str {
        "error parsing http request"
    }
    fn cause(&self) -> Option<&error::Error> {
        return None;
    }
}

trait BufReaderRet {
    fn read_line_ret(&mut self) -> Option<String>;
}

impl BufReaderRet for BufReader<TcpStream> {
    fn read_line_ret(&mut self) -> Option<String> {
        let mut buff = String::new();

        return match self.read_line(&mut buff) {
            Ok(_) => Option::Some(buff.trim().to_string()),
            Err(_) => Option::None
        }
    }
}

impl Request {
    fn create(mut reader: BufReader<TcpStream>) -> Result<Request> {
        let method: Method;
        let path: String;
        let version: String;

        if let Option::Some(req) = reader.read_line_ret() {
            let mut r = req.split(char::is_whitespace);

            if let Option::Some(m) = r.next() {
                method = match m {
                    "GET" => Method::GET,
                    "OPTIONS" => Method::OPTIONS,
                    "HEAD" => Method::HEAD,
                    v => Method::Other(v.to_string()),
                }
            } else {
                return Err(RequestParseError::new())
            }

            if let Option::Some(p) = r.next() {
                path = p.to_string();
            } else {
                return Err(RequestParseError::new())
            }

            if let Option::Some(v) = r.next() {
                version = v.to_string();
            } else {
                return Err(RequestParseError::new())
            }
        } else {
            return Err(RequestParseError::new())
        }

        let mut headers: HashMap<String, String> = HashMap::new();

        let prev = String::from("");

        while let Option::Some(line) = reader.read_line_ret() {
            if line == "" {
                break;
            } else {
                if line.starts_with(' ') || line.starts_with('\t') {
                    match headers.get_mut(prev.as_str()) {
                        Option::Some(v) => {
                            *v = format!("{}{}", *v, line);
                        }
                        Option::None => {}
                    }
                } else {
                    let mut b = line.splitn(2,':');
                    if let Some(key) = b.next() {
                        headers.insert(
                            key.to_string(),
                            match b.next() {
                                Some(v) => v,
                                None => "",
                            }.to_string().trim().to_string()
                        );
                    }
                }
            }
        }

        return std::result::Result::Ok(
            Request {
                method,
                version,
                path,
                headers,
                body: reader.into_inner().bytes(),
            }
        );
    }
}
impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Method: {} Path: {} Version: {}", self.method, self.path, self.version)
    }
}

pub fn handle_client(mut stream: TcpStream) {
    match stream.try_clone() {
        Ok(read_stream) => {
            let reader = BufReader::new(read_stream);
            let request = Request::create(reader);

            match request {
                Ok(r) => print!("{}\n{:?}\n", r, r.headers),
                Err(e) => eprintln!("{}", e)
            }

            match stream.write(b"HTTP/1.1 200 Ok\r\nContent-Type: text/plain\r\n\r\nHello World!\r\n") {
                Ok(_) => {
                    println!(" Success")
                },
                Err(e) => eprintln!("{}", e)
            }
        },
        Err(e) => eprintln!("{}", e)
    }

}
