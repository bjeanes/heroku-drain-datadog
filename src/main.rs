use std::{io,env};
use rouille::{Response,Server,router};

fn main() -> () {
    let port = match env::var("PORT") {
        Ok(val) => val,
        Err(_) => "0".to_string(),
    };

    let host = format!("0.0.0.0:{}", port);

    let server = Server::new(host, move |request| {
        rouille::log(request, io::stdout(), || {
            router!(request,
                (POST) (/logs/{_app: String}) => {
                    Response::text("OK")
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
