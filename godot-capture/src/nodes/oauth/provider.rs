use super::webserver::WebServer;
use super::{TokenError, TokenReceiver};
use std::sync::mpsc::sync_channel;
use std::thread;

pub struct OAuthProvider;

impl OAuthProvider {
    pub fn new() -> Self {
        OAuthProvider
    }

    pub fn provide<T, U>(&self, server: T, token_receiver: U) -> Result<String, TokenError>
    where
        T: WebServer + Send + Sync + 'static,
        U: TokenReceiver + Send + 'static,
    {
        let (send, token) = sync_channel(1);

        let server = server.token_sender(send);
        let state = token_receiver.state().ok_or(TokenError::NoStatePresent)?;
        let login_url = format!("https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=http://127.0.0.1:{}/capture/&response_type=token&state={}&scope=api",
                       server.port(), state);

        thread::spawn(move || {
            server.launch();
        });

        thread::spawn(move || {
            if let Ok((token, returned_state)) = token.recv() {
                token_receiver.token_received(token.as_str(), returned_state);
            }
        });

        Ok(login_url)
    }
}

#[cfg(test)]
mod tests {
    use super::super::mock_token_receiver::MockTokenReceiver;
    use super::*;
    use std::sync::mpsc::SyncSender;
    use std::sync::Arc;

    const STATE: i16 = 1;
    #[derive(Clone)]
    struct MocketWrapper {
        sync_sender: Option<SyncSender<(String, i16)>>,
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
        fn token_sender(mut self, sender: SyncSender<(String, i16)>) -> Self {
            self.sync_sender = Some(sender);
            self
        }

        fn launch(self) {
            self.sync_sender
                .map(|sender| sender.send(("token".to_string(), STATE)));
        }

        fn port(&self) -> u16 {
            self.port
        }
    }

    #[test]
    fn launches_webserver_on_provide() {
        let token_receiver = Arc::new(MockTokenReceiver::new(1));
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        let _url = server.provide(mock_server.clone(), Arc::clone(&token_receiver));
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert_eq!(Some("token".to_string()), token_receiver.received_token());
    }

    #[test]
    fn returns_url_for_login_on_provide_with_port() -> Result<(), TokenError> {
        let token_receiver = Arc::new(MockTokenReceiver::new(1));
        let server = OAuthProvider::new();
        let mut mock_server = MocketWrapper::new();
        mock_server.port = 10000;

        let url = server.provide(mock_server.clone(), Arc::clone(&token_receiver))?;

        assert!(url.starts_with("https://gitlab.com/oauth/authorize"));
        assert!(url.contains("&redirect_uri=http://127.0.0.1:10000/capture/"));
        Ok(())
    }

    #[test]
    fn includes_the_generated_state_in_the_url() -> Result<(), TokenError> {
        let state = 894;
        let token_receiver = Arc::new(MockTokenReceiver::new(state));
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        let url = server.provide(mock_server.clone(), Arc::clone(&token_receiver))?;

        assert!(url.contains(format!("&state={}", state).as_str()));
        Ok(())
    }

    #[test]
    fn does_not_provide_the_url_when_the_state_is_not_present() {
        let token_receiver = Arc::new(MockTokenReceiver::no_state_present());
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        let url = server.provide(mock_server.clone(), Arc::clone(&token_receiver));

        assert_eq!(Err(TokenError::NoStatePresent), url);
    }

    #[test]
    fn uses_the_state_from_the_webserver_when_saving_token() -> Result<(), TokenError> {
        let mismatched_state = STATE + 1;
        let token_receiver = Arc::new(MockTokenReceiver::new(mismatched_state));
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        server.provide(mock_server.clone(), Arc::clone(&token_receiver))?;
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert_eq!(Some(STATE), token_receiver.received_state());
        Ok(())
    }
}
