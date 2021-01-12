use super::TokenError;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use thiserror::Error;
use tokio::runtime::Runtime;
use url::Url;

pub trait WebServer {
    fn launch(self, callback: impl Fn(&str, i16) -> Result<(), TokenError> + 'static + Send + Sync);
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
async fn capture(
    callback: Arc<impl Fn(&str, i16) -> Result<(), TokenError> + 'static + Send + Sync>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    if req.uri().path() == "/Capture" {
        Ok(Response::new("Hello, World".into()))
    } else if req.uri().path() == "/save_token" {
        let params = url::form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
            .into_owned()
            .collect::<HashMap<String, String>>();

        let state = params
            .get("state")
            .and_then(|state| state.parse::<i16>().ok())
            .unwrap();

        callback(params.get("token").unwrap_or(&"".to_string()), state);

        Ok(Response::new(
            "Login Successful. Redirect To Capture App".into(),
        ))
    } else {
        let mut not_found = Response::default();
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        Ok(not_found)
    }
}

impl WebServer for HyperWebServer {
    fn port(&self) -> u16 {
        self.port
    }

    fn launch(
        self,
        callback: impl Fn(&str, i16) -> Result<(), TokenError> + 'static + Send + Sync,
    ) {
        std::thread::spawn(move || {
            let callback = Arc::new(callback);

            let make_svc = make_service_fn(move |_conn| {
                let callback = callback.clone();
                async {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        let callback = callback.clone();
                        capture(callback, req)
                    }))
                }
            });

            let mut rt = Runtime::new().unwrap(); // Do I want unwrap?
            rt.block_on(async {
                let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

                let server = Server::bind(&addr).serve(make_svc);

                if let Err(e) = server.await {
                    eprintln!("server error: {}", e);
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::Mutex;

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

    #[test]
    fn capture_calls_the_callback_on_save_token_with_the_params() {
        let url = "http://localhost:8000/save_token?token=token&state=1";
        let req = hyper::Request::builder()
            .method("POST")
            .uri(url)
            .body(Body::empty())
            .unwrap();

        let called = Arc::new(Mutex::new(RefCell::new(false)));
        let callback_called = called.clone();

        let callback = Arc::new(move |token: &str, state: i16| -> Result<(), TokenError> {
            assert_eq!("token", token);
            assert_eq!(1, state);
            *callback_called.lock().unwrap().borrow_mut() = true;
            Ok(())
        });

        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            capture(callback, req).await;
        });

        assert!(*called.lock().unwrap().borrow());
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
