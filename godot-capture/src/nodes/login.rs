use crate::oauth::{get_token, OAuthProvider, RocketWebServer};
use gdnative::api::OS;
use gdnative::prelude::*;

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
        let provider = OAuthProvider::new();
        let port = port_check::free_local_port();
        match RocketWebServer::builder().port(port).build() {
            Ok(rocket) => {
                let url = provider.provide(rocket);
                self.login_url = Some(url);
            }
            Err(err) => godot_error!("Error {:?} building rocket", err),
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
