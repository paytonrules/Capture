use crate::oauth::{login_site, OAuthServer, RocketWrapper};
use gdnative::prelude::*;
use std::sync::mpsc::Receiver;

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
            rocket: login_site::rocket(8080),
        };
        self.token_receiver = Some(server.start(rocket));
    }

    #[export]
    fn _process(&self, _owner: TRef<Node>, _delta: f64) {
        if let Some(token_receiver) = &self.token_receiver {
            if let Ok(_token) = token_receiver.try_recv() {
                godot_print!("token came back! (but lets not print it)");
            }
        }
    }
}
