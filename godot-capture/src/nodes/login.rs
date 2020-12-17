use crate::oauth::{create_state_generator, get_token, BuildError, OAuthProvider, RocketWebServer};
use gdnative::api::OS;
use gdnative::prelude::*;
use rand::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("Attempting to run Capture on unsupported platform")]
    UnsupportedPlatform,

    #[error("Error setting up OAuth {0}")]
    OAuthError(BuildError),
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Login {
    login_url: Option<String>,
}

#[methods]
impl Login {
    fn new(_owner: &Node) -> Self {
        Login { login_url: None }
    }

    #[export]
    fn _ready(&mut self, _owner: TRef<Node>) {
        let login_url = match OS::godot_singleton().get_name().to_string().as_str() {
            "OSX" => Ok(initialize_mac_oauth as fn() -> Result<String, Error>),
            "iOS" => Ok(initialize_ios_oauth as fn() -> Result<String, Error>),
            _ => Err(Error::UnsupportedPlatform),
        }
        .and_then(|f| f());

        match login_url {
            Ok(url) => self.login_url = Some(url),
            Err(err) => godot_error!("Error {:?} preparing login", err),
        };
    }

    #[export]
    fn _button_pressed(&self, _owner: TRef<Node>) {
        if let Some(login_url) = &self.login_url {
            OS::godot_singleton()
                .shell_open(login_url)
                .expect("should open");
        }
    }

    #[export]
    fn _process(&self, owner: TRef<Node>, _delta: f64) {
        if let Ok(_) = get_token() {
            self.token_received(owner)
        }
    }

    #[export]
    fn token_received(&self, owner: TRef<Node>) {
        owner
            .get_tree()
            .map(|tree| unsafe { tree.assume_safe() })
            .map(|tree| {
                tree.change_scene("res://CaptureNote.tscn")
                    .expect("Should change scene");
            });
    }
}

fn initialize_mac_oauth() -> Result<String, Error> {
    let provider = OAuthProvider::new();
    let port = port_check::free_local_port();
    let rocket = RocketWebServer::builder()
        .port(port)
        .build()
        .map_err(|err| Error::OAuthError(err))?;

    Ok(provider.provide(rocket, create_state_generator(random)))
}

fn initialize_ios_oauth() -> Result<String, Error> {
    Ok("https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=paytonrules.Capture://capture/&response_type=token&state=100&scope=api".to_string())
}
