use crate::oauth::{save_token, OAuthProvider, RocketWebServer};
use gdnative::api::OS;
use gdnative::prelude::*;
use std::sync::mpsc::Receiver;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Login {
    login_url: Option<String>,
    token_receiver: Option<Receiver<String>>,
}

#[methods]
impl Login {
    fn new(_owner: &Node) -> Self {
        Login {
            login_url: None,
            token_receiver: None,
        }
    }

    #[export]
    fn _ready(&mut self, _owner: TRef<Node>) {
        let provider = OAuthProvider::new();
        let port = port_check::free_local_port();
        match RocketWebServer::builder().port(port).build() {
            Ok(rocket) => {
                let (token_receiver, url) = provider.provide(rocket);
                self.token_receiver = Some(token_receiver);
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
        // biz_thing.check_for_token()
        //
        if let Some(token_receiver) = &self.token_receiver {
            if let Ok(token) = token_receiver.try_recv() {
                self.token_received(owner, token)
            }
        }
    }

    #[export]
    fn token_received(&self, owner: TRef<Node>, token: String) {
        owner
            .get_tree()
            .map(|tree| unsafe { tree.assume_safe() })
            .map(|tree| {
                save_token(token);
                tree.change_scene("res://CaptureNote.tscn")
                    .expect("Should change scene");
            });
    }
}
