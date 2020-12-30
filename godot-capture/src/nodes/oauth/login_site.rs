/*use rocket::config::{Config, Environment};
use rocket::request::Form;
use rocket::response::content::Html;
use rocket::State;
use std::sync::mpsc::SyncSender;
*/
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
/*
#[get("/capture")]
fn capture() -> Html<&'static str> {
    Html(LOGIN_SUCCESSFUL_PAGE)
}

#[derive(FromForm, Debug)]
struct Token {
    access_token: String,
    token_type: String,
    state: i16,
}

#[post("/save_token", data = "<token>")]
fn save_token(
    token_sender: State<SyncSender<(String, i16)>>,
    token: Form<Token>,
) -> Result<(), std::sync::mpsc::SendError<(String, i16)>> {
    token_sender.send((token.access_token.clone(), token.state))
}

pub fn rocket(port: u16) -> rocket::Rocket {
    let config = Config::build(Environment::Development)
        .address("127.0.0.1")
        .port(port)
        .unwrap();

    rocket::custom(config).mount("/", routes![capture, save_token])
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Request, Response, Server};
    use std::convert::Infallible;
    use std::net::SocketAddr;
    use std::sync::mpsc::sync_channel;
    use tokio::runtime::Runtime;
    use tokio::sync::oneshot;
    use ureq::get;

    #[test]
    fn capture_renders_a_simple_web_page() -> Result<(), Box<dyn std::error::Error>> {
        let mut rt = Runtime::new().unwrap();

        async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
            Ok(Response::new("Hello, World".into()))
        }

        let (sender, receiver) = oneshot::channel::<bool>();
        async fn shutdown_signal(receiver: oneshot::Receiver<bool>) {
            receiver.await.expect("couldn't shutdown");
        }

        rt.block_on(async {
            let join = tokio::task::spawn(async {
                let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
                let make_svc = make_service_fn(|_conn| async {
                    // service_fn converts our function into a `Service`
                    Ok::<_, Infallible>(service_fn(hello_world))
                });

                let server = Server::bind(&addr).serve(make_svc);

                let graceful = server.with_graceful_shutdown(shutdown_signal(receiver));

                // Run this server for... forever!
                if let Err(e) = graceful.await {
                    eprintln!("server error: {}", e);
                }
            });

            sender.send(true);

            join.await
        });
        Ok(())
    }

    /*
    #[test]
    fn posting_to_save_token_sends_the_token_and_state_to_the_channel(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (send, recv) = sync_channel::<(String, i16)>(1);
        let rocket = rocket(8080).manage(send);
        let client = Client::new(rocket)?;

        let response = client
            .post("/save_token")
            .body("access_token=token&state=200&token_type=ignore")
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(Ok(("token".to_string(), 200)), recv.recv());
        assert_eq!(Status::Ok, response.status());
        Ok(())
    }*/
}
