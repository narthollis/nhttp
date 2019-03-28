use std::net::{TcpListener};
use std::io::{Result as IoResult};

mod http_base;

fn main() -> IoResult<()> {
    let listener = TcpListener::bind("0.0.0.0:8080");

    return match listener {
        Result::Ok(l) => {
            for stream in l.incoming() {
                if let Ok(s) = stream {
                    http_base::handle_client(s);
                }
            }

            return Result::Ok(());
        },
        Result::Err(e) => Result::Err(e),
    };
}
