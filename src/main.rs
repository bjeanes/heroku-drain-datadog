use std::{io,env};
use std::io::Read;
use rouille::{Response,Request,Server,router};

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

fn handle_logs(request: &Request, _app: String) -> Response {
    let body = match request.data() {
        Some(b) => b,
        None => return Response::text("Internal Error").with_status_code(500),
    };

    let mut out = Vec::new();
    match body.take(BODY_LIMIT.saturating_add(1) as u64).read_to_end(&mut out) {
        Err(_) => return Response::text("Internal Error").with_status_code(500),
        _ => {},
    };

    let body = match String::from_utf8(out) {
        Ok(o) => o,
        _ => return Response::text("Internal Error").with_status_code(500),
    };

    println!("{}", body);

    Response::text("OK")
}
