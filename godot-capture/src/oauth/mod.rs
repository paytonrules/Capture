use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use thiserror::Error;
pub mod login_site;

pub trait WebServer {
    fn token_sender(self, sender: SyncSender<String>) -> Self;
    fn launch(self);
}

#[derive(Debug, PartialEq, Error)]
pub enum BuildError {
    #[error("No port was available to the builder, or the provided port was `None`")]
    NoPortProvided,
}

pub struct RocketWebServerBuilder {
    port: Option<u16>,
}

impl RocketWebServerBuilder {
    pub fn port(mut self, port: Option<u16>) -> Self {
        self.port = port;
        self
    }

    pub fn build(self) -> Result<RocketWebServer, BuildError> {
        self.port
            .map(|port| RocketWebServer {
                rocket: login_site::rocket(port),
            })
            .ok_or(BuildError::NoPortProvided)
    }
}

pub struct RocketWebServer {
    pub rocket: rocket::Rocket,
}

impl RocketWebServer {
    pub fn builder() -> RocketWebServerBuilder {
        RocketWebServerBuilder { port: None }
    }
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

    impl RocketWebServer {
        pub fn port(&self) -> u16 {
            self.rocket.config().port
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

    #[test]
    fn build_rocket_webserver_with_port_provider() -> Result<(), BuildError> {
        let wrapper = RocketWebServer::builder().port(Some(9001)).build()?;

        assert_eq!(9001, wrapper.port());
        Ok(())
    }

    #[test]
    fn fail_to_build_without_a_port() {
        let wrapper = RocketWebServer::builder().build();

        if let Err(error) = wrapper {
            assert_eq!(BuildError::NoPortProvided, error);
        } else {
            assert!(false, "Did not return an error as expected");
        }
    }
}
