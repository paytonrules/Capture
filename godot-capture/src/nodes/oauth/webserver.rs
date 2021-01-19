use super::TokenError;
use hyper::service::{make_service_fn, service_fn};
use hyper::Method;
use hyper::{Body, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::{
    oneshot::{self, Sender},
    Mutex,
};

lazy_static! {
    static ref SHUTDOWN_TX: Arc<Mutex<Option<Sender<()>>>> = <_>::default();
}

const LOGIN_SUCCESSFUL_PAGE: &'static str = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN">

<html>
<head>

  <script type="text/javascript">
    document.addEventListener("DOMContentLoaded", (event) => {
        const hashAsParams = new URLSearchParams(
            window.location.hash.substr(1)
        );
        fetch("/save_token", {
            method: 'POST',
            body: hashAsParams
        });
    });
  </script>

  <title></title>
</head>

<body>
  <p>Login Successful, return to the Capture app.</p>
</body>
</html>"#;

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
async fn router(
    req: Request<Body>,
    callback: Arc<impl Fn(&str, i16) -> Result<(), TokenError> + 'static + Send + Sync>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/Capture") => Ok(Response::new(LOGIN_SUCCESSFUL_PAGE.into())),
        (&Method::POST, "/save_token") => {
            let params = url::form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
                .into_owned()
                .collect::<HashMap<String, String>>();

            let state = params
                .get("state")
                .and_then(|state| state.parse::<i16>().ok())
                .unwrap();

            if let Ok(_) = callback(params.get("token").unwrap_or(&"".to_string()), state) {
                if let Some(sender) = SHUTDOWN_TX.lock().await.take() {
                    sender.send(());
                }
            }

            Ok(Response::new(
                "Login Successful. Redirect To Capture App".into(),
            ))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
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
                        router(req, callback)
                    }))
                }
            });
            let mut rt = Runtime::new().unwrap(); // Do I want unwrap?
            rt.block_on(async {
                let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

                let server = Server::bind(&addr).serve(make_svc);
                let (sender, receiver) = oneshot::channel::<()>();
                SHUTDOWN_TX.lock().await.replace(sender);

                let graceful = server.with_graceful_shutdown(async {
                    receiver.await.ok();
                });

                if let Err(e) = graceful.await {
                    eprintln!("server error: {}", e);
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::cell::RefCell;
    use std::sync::Mutex;

    use super::*;
    use ureq::{get, post};

    #[test]
    fn webserver_is_built_with_provided_port() {
        let webserver = HyperWebServer::new(9001);

        assert_eq!(9001, webserver.port());
    }

    #[test]
    #[serial(webserver)]
    fn launch_starts_a_server_with_a_capture_route() {
        let (webserver, port) = create_webserver();
        let url = format!("http://localhost:{}/Capture", port);

        webserver.launch(|_first, _second| Ok(()));

        let r = get(&url).call();
        assert!(r.ok());
    }

    #[test]
    #[serial(webserver)]
    fn launch_doesnt_respond_to_the_root() {
        let (webserver, port) = create_webserver();
        let url = format!("http://localhost:{}", port);

        webserver.launch(|_first, _second| Ok(()));

        let r = get(&url).call();
        assert!(r.error());
    }

    #[test]
    #[serial(webserver)]
    fn webserver_shutsdown_when_correct_token_and_state_are_sent() {
        let (webserver, port) = create_webserver();
        let url = format!("http://localhost:{}/save_token?token=token&state=1", port);

        webserver.launch(|_first, _second| Ok(()));

        let r = post(&url).call();
        assert!(r.ok());
        let r = post(&url).call();
        assert!(r.error());
    }

    fn create_callback_with(
        cb: impl Fn(&str, i16) -> Result<(), TokenError> + 'static + Send + Sync,
    ) -> (
        Arc<impl Fn(&str, i16) -> Result<(), TokenError> + 'static + Send + Sync>,
        Arc<Mutex<RefCell<bool>>>,
    ) {
        let called = Arc::new(Mutex::new(RefCell::new(false)));
        let callback_called = called.clone();

        let wrapped_callback = Arc::new(move |token: &str, state: i16| -> Result<(), TokenError> {
            *callback_called.lock().unwrap().borrow_mut() = true;
            cb(token, state)
        });
        (wrapped_callback, called)
    }

    fn assert_called(called: Arc<Mutex<RefCell<bool>>>) {
        assert!(*called.lock().unwrap().borrow());
    }

    fn assert_not_called(called: Arc<Mutex<RefCell<bool>>>) {
        assert_eq!(false, *called.lock().unwrap().borrow());
    }

    #[derive(Debug)]
    enum TestError {
        HyperError(hyper::Error),
        TokioError(tokio::io::Error),
        FromUtf8Error(std::string::FromUtf8Error),
        OneshotError(tokio::sync::oneshot::error::TryRecvError),
    }

    impl From<tokio::sync::oneshot::error::TryRecvError> for TestError {
        fn from(err: tokio::sync::oneshot::error::TryRecvError) -> Self {
            TestError::OneshotError(err)
        }
    }

    type TestResult = Result<(), TestError>;

    fn run_router(
        callback: Arc<impl Fn(&str, i16) -> Result<(), TokenError> + 'static + Send + Sync>,
        req: Request<Body>,
    ) -> Result<Response<Body>, TestError> {
        let mut rt = Runtime::new().map_err(|err| TestError::TokioError(err))?;
        rt.block_on(async { router(req, callback).await })
            .map_err(|err| TestError::HyperError(err))
    }

    fn response_as_string(response: &mut Response<Body>) -> Result<String, TestError> {
        let mut runtime = Runtime::new().map_err(|err| TestError::TokioError(err))?;
        let bytes = runtime
            .block_on(hyper::body::to_bytes(response.body_mut()))
            .map_err(|err| TestError::HyperError(err))?;

        String::from_utf8(bytes.into_iter().collect()).map_err(|err| TestError::FromUtf8Error(err))
    }

    #[test]
    fn router_renders_the_default_html_page_on_capture() -> TestResult {
        let url = "http://localhost:8000/Capture";
        let req = hyper::Request::builder()
            .method("GET")
            .uri(url)
            .body(Body::empty())
            .unwrap();

        let (callback, _) = create_callback_with(move |token, state| Ok(()));

        let mut response = run_router(callback, req)?;
        assert_eq!(
            LOGIN_SUCCESSFUL_PAGE.to_string(),
            response_as_string(&mut response)?
        );

        Ok(())
    }

    #[test]
    fn router_calls_the_callback_on_save_token_with_the_params() -> TestResult {
        let url = "http://localhost:8000/save_token?token=token&state=1";
        let req = hyper::Request::builder()
            .method("POST")
            .uri(url)
            .body(Body::empty())
            .unwrap();

        let (callback, called) = create_callback_with(|token, state| {
            assert_eq!("token", token);
            assert_eq!(1, state);
            Ok(())
        });

        run_router(callback, req)?;

        assert_called(called);
        Ok(())
    }

    #[test]
    fn router_does_not_call_the_callback_on_save_token_as_a_get() -> TestResult {
        let (sender, _receiver) = oneshot::channel::<bool>();
        let url = "http://localhost:8000/save_token?token=token&state=1";
        let req = hyper::Request::builder()
            .method("GET")
            .uri(url)
            .body(Body::empty())
            .unwrap();

        let (callback, called) = create_callback_with(move |token, state| Ok(()));

        run_router(callback, req)?;

        assert_not_called(called);
        Ok(())
    }
    // Test capture function sends the 'channel' oneshot on success
    // Test webserver shuts down gracefully when the callback fn returns Ok

    fn create_webserver() -> (HyperWebServer, u16) {
        let port = port_check::free_local_port().expect("Could not find free port!");
        let webserver = HyperWebServer::new(port);

        (webserver, port)
    }
}
