use rocket::config::{Config, Environment};
use rocket::request::Form;
use rocket::response::content::Html;
use rocket::State;
use std::sync::mpsc::SyncSender;

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

pub fn rocket(port: u16) -> rocket::Rocket {
    let config = Config::build(Environment::Development)
        .address("127.0.0.1")
        .port(port)
        .unwrap();

    rocket::custom(config).mount("/", routes![capture, save_token])
}
