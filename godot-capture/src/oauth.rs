use gdnative::prelude::*;
use rocket::config::{Config, Environment};
use rocket::request::Form;
use rocket::response::content::Html;
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
fn capture() -> Html<&'static str> {
    Html(LOGIN_SUCCESSFUL_PAGE)
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

trait WebServer {
    fn manage(self, state: SyncSender<String>) -> Self;
    fn launch(self);
}

struct RocketWrapper {
    rocket: rocket::Rocket,
}

impl WebServer for RocketWrapper {
    fn manage(self, state: SyncSender<String>) -> Self {
        RocketWrapper {
            rocket: self.rocket.manage(state),
        }
    }
    fn launch(self) {
        self.rocket.launch();
    }
}

// TODO rename (OauthProvider? Something that's less "serverish")
struct OAuthServer;

impl OAuthServer {
    fn new() -> Self {
        OAuthServer
    }

    fn start<T: WebServer + Send + Sync + 'static>(&self, server: T) -> Receiver<String> {
        let (send, recv) = sync_channel(1);

        let server = server.manage(send);

        // Give it a port

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
        let rocket = RocketWrapper {
            rocket: rocket(8080),
        };
        self.token_receiver = Some(server.start(rocket));
    }

    #[export]
    fn _process(&self, _owner: TRef<Node>, _delta: f64) {
        if let Some(token_receiver) = &self.token_receiver {
            if let Ok(token) = token_receiver.try_recv() {
                godot_print!("token! {}", token);
                // TODO fire a signal 'token received'
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;
    use std::sync::Arc;
    use std::sync::RwLock;

    #[derive(Clone)]
    struct MocketWrapper {
        sync_sender: Option<SyncSender<String>>,
    }

    impl MocketWrapper {
        fn new() -> Self {
            MocketWrapper { sync_sender: None }
        }
    }

    impl WebServer for MocketWrapper {
        fn manage(mut self, state: SyncSender<String>) -> Self {
            self.sync_sender = Some(state);
            self
        }

        fn launch(self) {
            self.sync_sender
                .map(|sender| sender.send("token".to_string()));
        }
    }

    #[test]
    fn test_launches_webserver_on_start() {
        let server = OAuthServer::new();
        let mock_server = MocketWrapper::new();

        let receiver = server.start(mock_server.clone());
        let token = receiver.recv_timeout(std::time::Duration::from_millis(10));

        assert_eq!("token".to_string(), token.unwrap());
    }

    #[test]
    fn test_payton_understands_arc_and_rwlock() {
        let first = Arc::new(RwLock::new(1));
        let second = first.clone();

        let join_handle = thread::spawn(move || {
            let mut thing = second.write().unwrap();
            *thing = 2;
        });
        join_handle.join();

        assert_eq!(*first.read().unwrap(), 2);
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
