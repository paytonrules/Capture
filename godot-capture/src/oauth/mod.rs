use gdnative::prelude::*;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
mod login_site;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
