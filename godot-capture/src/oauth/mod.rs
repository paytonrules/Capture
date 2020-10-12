use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use thiserror::Error;
pub mod login_site;

pub trait WebServer {
    fn token_sender(self, sender: SyncSender<String>) -> Self;
    fn launch(self);
    fn port(&self) -> u16;
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
    fn port(&self) -> u16 {
        self.rocket.config().port
    }
}

pub struct OAuthProvider;

impl OAuthProvider {
    pub fn new() -> Self {
        OAuthProvider
    }

    pub fn provide<T: WebServer + Send + Sync + 'static>(
        &self,
        server: T,
    ) -> (Receiver<String>, String) {
        let (send, recv) = sync_channel(1);

        let server = server.token_sender(send);
        let login_url = format!("https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=http://127.0.0.1:{}/capture/&response_type=token&state=100&scope=api",
                       server.port());

        thread::spawn(move || {
            server.launch();
        });
        (recv, login_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MocketWrapper {
        sync_sender: Option<SyncSender<String>>,
        port: u16,
    }

    impl MocketWrapper {
        fn new() -> Self {
            MocketWrapper {
                sync_sender: None,
                port: 0,
            }
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

        fn port(&self) -> u16 {
            self.port
        }
    }

    #[test]
    fn launches_webserver_on_provide() {
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        let (receiver, _url) = server.provide(mock_server.clone());
        let token = receiver.recv_timeout(std::time::Duration::from_millis(10));

        assert_eq!("token".to_string(), token.unwrap());
    }

    #[test]
    fn returns_url_for_login_on_provide_with_port() {
        let server = OAuthProvider::new();
        let mut mock_server = MocketWrapper::new();
        mock_server.port = 10000;

        let (_receiver, url) = server.provide(mock_server.clone());

        assert!(url.starts_with("https://gitlab.com/oauth/authorize"));
        assert!(url.contains("&redirect_uri=http://127.0.0.1:10000/capture/"))
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
