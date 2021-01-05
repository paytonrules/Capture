use super::TokenError;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::net::SocketAddr;
use std::sync::mpsc::SyncSender;
use thiserror::Error;
use tokio::runtime::Runtime;

pub trait WebServer {
    fn launch(self, callback: impl FnOnce(&str, i16) -> Result<(), TokenError> + 'static + Send);
    fn port(&self) -> u16;
}

pub struct HyperWebServer {
    port: u16,
}

impl HyperWebServer {
    pub fn new(port: u16) -> Self {
        HyperWebServer { port }
    }
}

impl WebServer for HyperWebServer {
    fn port(&self) -> u16 {
        self.port
    }

    fn launch(self, callback: impl FnOnce(&str, i16) -> Result<(), TokenError> + 'static + Send) {
        std::thread::spawn(move || {
            async fn capture(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
                if req.uri().path() == "/Capture" {
                    Ok(Response::new("Hello, World".into()))
                } else {
                    let mut not_found = Response::default();
                    *not_found.status_mut() = StatusCode::NOT_FOUND;
                    Ok(not_found)
                }
            }

            let make_svc =
                make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(capture)) });

            let mut rt = Runtime::new().unwrap(); // Do I want unwrap?
            rt.block_on(async {
                let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

                let server = Server::bind(&addr).serve(make_svc);

                if let Err(e) = server.await {
                    eprintln!("server error: {}", e);
                }
            });
        });

        // make call to callback
        // if success / done
        // if fail - restart
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ureq::get;

    #[test]
    fn webserver_is_built_with_provided_port() {
        let webserver = HyperWebServer::new(9001);

        assert_eq!(9001, webserver.port());
    }

    #[test]
    fn launch_starts_a_server_with_a_capture_route() {
        let (webserver, port) = create_webserver();
        let url = format!("http://localhost:{}/Capture", port);

        webserver.launch(|_first, _second| Ok(()));

        let r = get(&url).call();
        assert!(r.ok());
    }

    #[test]
    fn launch_doesnt_respond_to_the_root() {
        let (webserver, port) = create_webserver();
        let url = format!("http://localhost:{}", port);

        webserver.launch(|_first, _second| Ok(()));

        let r = get(&url).call();
        assert!(r.error());
    }

    // Test capture renders the right login page
    // Test posting the state and token passes them through to the callback
    // Test capture function sends the 'channel' oneshot on success
    // Test webserver shuts down gracefully when the callback fn returns Ok

    fn create_webserver() -> (HyperWebServer, u16) {
        let port = port_check::free_local_port().expect("Could not find free port!");
        let webserver = HyperWebServer::new(port);

        (webserver, port)
    }
}
