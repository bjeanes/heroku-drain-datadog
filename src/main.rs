use std::{io, env};
use std::io::Read;
use rouille::{
    Request,
    Response,
    Server,
    assert_or_400,
    router,
};

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
    }).unwrap();

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
        ( request.header("Content-Length").unwrap().parse::<usize>().unwrap() ) <= BODY_LIMIT
    );

    match request.header("Content-Type") {
        Some("application/logplex-1") => {
            let body = match request.data() {
                Some(b) => b,
                None => return internal_error(),
            };

            let mut out = Vec::new();
            match body.take(BODY_LIMIT.saturating_add(1) as u64).read_to_end(&mut out) {
                Err(_) => return internal_error(),
                _ => {},
            };

            if out.len() > BODY_LIMIT {
                return Response::empty_400()
            }

            let body = match String::from_utf8(out) {
                Ok(o) => o,
                _ => return internal_error(),
            };

            println!("{}", body);

            Response::text("OK")
        },
        Some(ct) => {
            Response::text(format!("Unexpected Content-Type: {}", ct)).with_status_code(400)
        },
        _ => {
            Response::text("Missing Content-Type").with_status_code(400)
        }
    }

}
