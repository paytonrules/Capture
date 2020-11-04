use super::login_site;
use std::sync::mpsc::SyncSender;
use thiserror::Error;

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

#[cfg(test)]
mod tests {
    use super::*;

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
