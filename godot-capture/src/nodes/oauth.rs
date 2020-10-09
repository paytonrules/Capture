use crate::oauth::{OAuthProvider, RocketWebServer};
use gdnative::prelude::*;
use port_check;
use std::sync::mpsc::Receiver;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct OAuthValidation {
    token_receiver: Option<Receiver<String>>,
}

#[methods]
impl OAuthValidation {
    fn new(_owner: &Node) -> Self {
        OAuthValidation {
            token_receiver: None,
        }
    }

    #[export]
    fn _ready(&mut self, _owner: TRef<Node>) {
        let provider = OAuthProvider::new();
        let port = port_check::free_local_port();
        match RocketWebServer::builder().port(port).build() {
            Ok(rocket) => self.token_receiver = Some(provider.provide(rocket)),
            Err(err) => godot_print!("Error {:?} building rocket", err),
        };
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
