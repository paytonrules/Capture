use gdnative::prelude::*;
use rocket::config::{Config, Environment};
use rocket::request::Form;
use rocket::State;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;

// Take a mock object in the Oauth struct
// Choose an available port rather than being hard coded to 8080
//

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

#[get("/capture")]
fn capture() -> rocket::response::content::Html<&'static str> {
    rocket::response::content::Html(LOGIN_SUCCESSFUL_PAGE)
}

#[derive(FromForm, Debug)]
struct Token {
    access_token: String,
    token_type: String,
    state: String,
}

#[post("/save_token", data = "<token>")]
fn save_token(
    token_sender: State<SyncSender<String>>,
    token: Form<Token>,
) -> Result<(), std::sync::mpsc::SendError<String>> {
    token_sender.send(token.access_token.clone())
}

fn rocket(port: u16) -> rocket::Rocket {
    let config = Config::build(Environment::Development)
        .address("127.0.0.1")
        .port(port)
        .unwrap();

    rocket::custom(config).mount("/", routes![capture, save_token])
}

// TODO rename (OauthProvider? Something that's less "serverish")
struct OAuthServer;

impl OAuthServer {
    fn new() -> Self {
        OAuthServer
    }

    fn start(&self, server: rocket::Rocket) -> Receiver<String> {
        let (send, recv) = sync_channel(1);

        let server = server.manage(send);

        thread::spawn(move || {
            server.launch();
        });
        recv
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Listener {
    token_receiver: Option<Receiver<String>>,
}

#[methods]
impl Listener {
    fn new(_owner: &Node) -> Self {
        Listener {
            token_receiver: None,
        }
    }

    #[export]
    fn _ready(&mut self, _owner: TRef<Node>) {
        let server = OAuthServer::new();
        let rocket = rocket(8080);
        self.token_receiver = Some(server.start(rocket));
    }

    #[export]
    fn _process(&self, _owner: TRef<Node>, _delta: f64) {
        if let Some(token_receiver) = &self.token_receiver {
            if let Ok(token) = token_receiver.try_recv() {
                godot_print!("token! {}", token);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;

    #[test]
    fn test_spawns_webserver_on_start() -> std::io::Result<()> {
        let server = OAuthServer::new();
        let _tokench = server.start(rocket(8080));

        let res = ureq::get(
            "http://127.0.0.1:8080/capture/#access_token=token&token_type=Bearer&state=100",
        )
        .call();

        assert_eq!(200, res.status());
        Ok(())
    }

    #[test]
    fn rocket_constructor_uses_passed_in_port() {
        let rocket = rocket(8000);

        assert_eq!(8000, rocket.config().port);
    }

    #[test]
    fn capture_renders_a_simple_web_page() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(rocket(8080))?;

        let mut response = client.get("/capture").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::HTML));
        assert!(response
            .body_string()
            .ok_or("Error getting html body")?
            .contains("Login Successful"));

        Ok(())
    }

    #[test]
    fn posting_to_save_token_sends_the_token_to_the_channel(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (send, recv) = sync_channel(1);
        let rocket = rocket(8080).manage(send);
        let client = Client::new(rocket)?;

        let response = client
            .post("/save_token")
            .body("access_token=token&state=ignore&token_type=ignore")
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(Ok("token".to_string()), recv.recv());
        assert_eq!(Status::Ok, response.status());
        Ok(())
    }
}
