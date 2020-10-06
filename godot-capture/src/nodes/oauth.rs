use crate::oauth::{login_site, OAuthProvider, PortProvider, RocketWebServer};
use gdnative::prelude::*;
use std::sync::mpsc::Receiver;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct OAuthValidation {
    token_receiver: Option<Receiver<String>>,
}

struct DummyPortProvider;

impl PortProvider for DummyPortProvider {
    fn provide(self) -> u16 {
        8080
    }
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
        let rocket = RocketWebServer {
            rocket: login_site::rocket(8080),
        };
        self.token_receiver = Some(provider.provide(rocket));
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
