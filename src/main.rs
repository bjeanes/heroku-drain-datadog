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

            if out.len() > BODY_LIMIT {
                return Response::empty_400();
            }

            let mut body = BufReader::new(&out[..]);

            println!("{:?}", &out);
            println!("{:?}", str::from_utf8(&out));

            let mut buf: Vec<u8> = Vec::new();
            let mut buf2: Vec<u8> = Vec::new();
            let message = match body.read_until(b' ', &mut buf) {
                Err(_) => return internal_error(),
                Ok(matched_bytes) => {
                    let message_size: u64 = {
                        println!("body BufReader: {:?}", &body);
                        println!("buffer contents: {:?}", &buf);
                        let bytes_string = &buf[0..matched_bytes];
                        println!("matched_bytes: {:?}", bytes_string);
                        match str::from_utf8(bytes_string) {
                            Ok(s) => match s.trim().parse::<u64>() {
                                Ok(size) => {
                                    println!("message size: {}", size);
                                    size
                                }
                                _ => return internal_error(),
                            },
                            _ => return internal_error(),
                        }
                    };

                    {
                        let mut take = body.take(message_size);
                        println!("{:?}", take);
                        match take.read_to_end(&mut buf2) {
                            Ok(_) => match &str::from_utf8(&buf2) {
                                Ok(s) => match s.parse::<LogplexMessage>() {
                                    Ok(msg) => msg,
                                    _ => return internal_error(),
                                },
                                _ => return internal_error(),
                            },
                            _ => return internal_error(),
                        }
                    }
                }
            };

            println!("{:?}", message);

            // let mut messages = body.lines().map(|line| {
            //     println!("{:?}", line);
            //     line.parse::<LogplexMessage>()
            // });
            //
            // println!("{:?}", messages.next());

            Response::empty_204()
        }
        Some(ct) => {
            Response::text(format!("Unexpected Content-Type: {}", ct)).with_status_code(400)
        }
        _ => Response::text("Missing Content-Type").with_status_code(400),
    }
}
