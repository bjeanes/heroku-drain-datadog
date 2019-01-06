use rouille::{assert_or_400, router, Request, Response, Server};
use std::io::{BufRead, BufReader, Read};
use std::{env, io, str};
use syslog_heroku::Message as LogplexMessage;

fn main() -> () {
    let host = match env::var("PORT") {
        Ok(port) => format!("0.0.0.0:{}", port),
        _ => "0.0.0.0:0".to_string(),
    };

    let server = Server::new(host, move |request| {
        rouille::log(request, io::stdout(), || {
            router!(request,
                (POST) (/logs/{app: String}) => {
                    handle_logs(&request, app)
                },

                _ => {
                    Response::empty_404()
                },
            )
        })
    })
    .unwrap();

    {
        let addr = server.server_addr();
        println!("Listening on http://{}:{}", addr.ip(), addr.port());
    }

    server.run();
}

const BODY_LIMIT: usize = 1024 * 1024;

fn internal_error() -> Response {
    Response::text("Internal Error").with_status_code(500)
}
fn internal_error_with_message(body: &str) -> Response {
    Response::text(format!("Internal Error: {}", body)).with_status_code(500)
}

fn handle_logs(request: &Request, _app: String) -> Response {
    assert_or_400!(
        (request
            .header("Content-Length")
            .unwrap()
            .parse::<usize>()
            .unwrap())
            <= BODY_LIMIT
    );

    match request.header("Content-Type") {
        Some("application/logplex-1") => {
            let body = match request.data() {
                Some(b) => b,
                None => return internal_error(),
            };

            let mut out = Vec::new();
            match body
                .take(BODY_LIMIT.saturating_add(1) as u64)
                .read_to_end(&mut out)
            {
                Err(_) => return internal_error(),
                _ => {}
            };

            println!("Request body size: {}", out.len());

            if out.len() > BODY_LIMIT {
                return Response::empty_400();
            }

            let mut body = BufReader::new(&out[..]);

            loop {
                match next_message(&mut body) {
                    Ok(message) => {
                        println!("Message: {:?}", message);
                    }
                    Err(ParseErr::NoMoreMessages) => return Response::empty_204(),
                    Err(e) => return internal_error_with_message(&format!("{:?}", e)),
                }
            }
        }
        Some(ct) => {
            Response::text(format!("Unexpected Content-Type: {}", ct)).with_status_code(400)
        }
        _ => Response::text("Missing Content-Type").with_status_code(400),
    }
}

#[derive(Debug)]
enum ParseErr {
    FailedToReadMessageSize,
    FailedToParseMessageSize,
    GenericError,
    NoMoreMessages,
}
fn next_message(body: &mut BufRead) -> Result<LogplexMessage, ParseErr> {
    let mut message_size_buffer: Vec<u8> = Vec::new();
    let mut message_buffer: Vec<u8> = Vec::new();
    match body.read_until(b' ', &mut message_size_buffer) {
        Err(_) => return Err(ParseErr::FailedToReadMessageSize),
        Ok(matched_bytes) => {
            let message_size: u64 = {
                let bytes_string = &message_size_buffer[0..matched_bytes];
                match str::from_utf8(bytes_string) {
                    Ok(s) => match s.trim().parse::<u64>() {
                        Ok(size) => {
                            println!("message size: {}", size);
                            size
                        }
                        Err(_) => {
                            if s.trim() == "" {
                                return Err(ParseErr::NoMoreMessages);
                            } else {
                                return Err(ParseErr::FailedToParseMessageSize);
                            }
                        }
                    },
                    _ => return Err(ParseErr::FailedToParseMessageSize),
                }
            };

            let mut take = body.take(message_size);
            match take.read_to_end(&mut message_buffer) {
                Ok(_) => match &str::from_utf8(&message_buffer) {
                    Ok(s) => match s.parse::<LogplexMessage>() {
                        Ok(msg) => Ok(msg),
                        _ => return Err(ParseErr::GenericError),
                    },
                    _ => return Err(ParseErr::GenericError),
                },
                _ => return Err(ParseErr::GenericError),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::next_message;
    use std::io::BufReader;
    use stringreader::StringReader;

    #[test]
    fn it_parses_messages_from_bufreader() {
        let sample_body = include_str!("../test/sample-body");
        let mut body = BufReader::new(StringReader::new(sample_body));

        let msg1 = next_message(&mut body);
        let msg2 = next_message(&mut body);
        let msg3 = next_message(&mut body);

        assert!(msg1.is_ok());
        assert!(msg2.is_ok());
        assert!(!msg3.is_ok());

        assert_eq!(msg1.unwrap().msg, "State changed from starting to up\r");
        assert_eq!(
            msg2.unwrap().msg,
            "Starting process with command `bundle exec rackup config.ru -p 24405`\r"
        );
    }
}
