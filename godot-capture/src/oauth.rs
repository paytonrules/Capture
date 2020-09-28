use gdnative::prelude::*;
use rocket::config::{Config, Environment};
use rocket::request::Form;
use rocket::State;
use rocket_contrib::templates::Template;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::{collections::HashMap, thread};

#[get("/capture")]
fn capture() -> Template {
    let context: HashMap<String, String> = HashMap::new();
    Template::render("capture", &context)
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

fn rocket(port: u16, sender: SyncSender<String>) -> rocket::Rocket {
    let config = Config::build(Environment::Development)
        .address("127.0.0.1")
        .port(port)
        .unwrap();

    rocket::custom(config)
        .attach(Template::fairing())
        .manage(sender)
        .mount("/", routes![capture, save_token])
}

// TODO rename (OauthProvider? Something that's less "serverish")
struct OAuthServer;

impl OAuthServer {
    fn new() -> Self {
        OAuthServer
    }

    fn start(&self) -> Receiver<String> {
        let (send, recv) = sync_channel(1);

        let server = rocket(8080, send);

        thread::spawn(move || {
            server.launch();
        });
        recv
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Listener;

#[methods]
impl Listener {
    fn new(_owner: &Node) -> Self {
        Listener
    }

    #[export]
    fn _ready(&self, _owner: TRef<Node>) {
        let server = OAuthServer::new();
        let _tokench = server.start();
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
        let _tokench = server.start();

        let res = ureq::get(
            "http://127.0.0.1:8080/capture/#access_token=token&token_type=Bearer&state=100",
        )
        .call();

        assert_eq!(200, res.status());
        Ok(())
    }

    // Ensure we hit a valid port
    // shutdown
    // token expiration

    #[test]
    fn rocket_constructor_uses_passed_in_port() {
        let (send, _recv) = sync_channel(1);
        let rocket = rocket(8000, send);

        assert_eq!(8000, rocket.config().port);
    }

    #[test]
    fn capture_renders_a_simple_web_page() -> Result<(), Box<dyn std::error::Error>> {
        let (send, _recv) = sync_channel(1);
        let client = Client::new(rocket(8080, send))?;

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
        let client = Client::new(rocket(8080, send))?;

        let response = client
            .post("/save_token")
            .body("access_token=token&state=ignore&token_type=ignore")
            .header(ContentType::Form)
            .dispatch();

        assert_eq!(Ok("token".to_string()), recv.recv());
        assert_eq!(Status::Ok, response.status());
        Ok(())
    }
    // test state has matches
}
