use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
pub mod login_site;

pub trait WebServer {
    fn token_sender(self, sender: SyncSender<String>) -> Self;
    fn launch(self);
}

pub struct RocketWebServer {
    pub rocket: rocket::Rocket,
}

impl WebServer for RocketWebServer {
    fn token_sender(self, sender: SyncSender<String>) -> Self {
        RocketWebServer {
            rocket: self.rocket.manage(sender),
        }
    }
    fn launch(self) {
        self.rocket.launch();
    }
}

pub trait PortProvider {
    fn provide(self) -> u16;
}

pub struct OAuthProvider;

impl OAuthProvider {
    pub fn new() -> Self {
        OAuthProvider
    }

    pub fn provide<T: WebServer + Send + Sync + 'static>(&self, server: T) -> Receiver<String> {
        let (send, recv) = sync_channel(1);

        let server = server.token_sender(send);

        // Give it a port

        thread::spawn(move || {
            server.launch();
        });
        recv
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
        fn token_sender(mut self, sender: SyncSender<String>) -> Self {
            self.sync_sender = Some(sender);
            self
        }

        fn launch(self) {
            self.sync_sender
                .map(|sender| sender.send("token".to_string()));
        }
    }

    struct StubbedPortProvider {
        port: u16,
    }

    impl StubbedPortProvider {
        fn new(port: u16) -> Self {
            StubbedPortProvider { port }
        }
    }

    impl PortProvider for StubbedPortProvider {
        fn provide(self) -> u16 {
            self.port
        }
    }

    #[test]
    fn launches_provider_on_start() {
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        let receiver = server.provide(mock_server.clone());
        let token = receiver.recv_timeout(std::time::Duration::from_millis(10));

        assert_eq!("token".to_string(), token.unwrap());
    }
}
