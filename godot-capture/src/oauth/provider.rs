use super::save_token;
use super::webserver::WebServer;
use super::TokenReceiver;
use std::sync::mpsc::sync_channel;
use std::thread;

pub struct OAuthProvider;

impl OAuthProvider {
    pub fn new() -> Self {
        OAuthProvider
    }

    pub fn provide<T, U>(&self, server: T, token_receiver: U) -> String
    where
        T: WebServer + Send + Sync + 'static,
        U: TokenReceiver + Send + 'static,
    {
        let (send, token) = sync_channel(1);

        let server = server.token_sender(send);
        let state = 1;
        let login_url = format!("https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=http://127.0.0.1:{}/capture/&response_type=token&state={}&scope=api",
                       server.port(), state);

        thread::spawn(move || {
            server.launch();
        });

        thread::spawn(move || {
            if let Ok((token, returned_state)) = token.recv() {
                token_receiver.token_received(token.as_str(), 30480);
            }
        });

        login_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::*;
    use serial_test::serial;
    use std::cell::RefCell;
    use std::sync::mpsc::SyncSender;
    use std::sync::Arc;
    use std::sync::Mutex;

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

    struct MockTokenReceiver {
        state: i16,
        received_token: Mutex<RefCell<Option<String>>>,
        received_state: Mutex<RefCell<Option<i16>>>,
    }

    impl MockTokenReceiver {
        fn new(state: i16) -> Self {
            MockTokenReceiver {
                state,
                received_token: Mutex::new(RefCell::new(None)),
                received_state: Mutex::new(RefCell::new(None)),
            }
        }

        fn received_token(&self) -> Option<String> {
            (*self.received_token.lock().unwrap().borrow()).clone()
        }
    }

    impl TokenReceiver for Arc<MockTokenReceiver> {
        fn state(&self) -> Option<i16> {
            Some(self.state)
        }

        fn token_received(&self, token: &str, state: i16) -> Result<(), TokenError> {
            *self.received_token.lock().unwrap().borrow_mut() = Some(token.to_string());
            *self.received_state.lock().unwrap().borrow_mut() = Some(state);
            Ok(())
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
    /*
    #[test]
    #[serial(accesses_token)]
    fn returns_url_for_login_on_provide_with_port() {
        let state_gen = create_state_generator(|| STATE);
        let server = OAuthProvider::new();
        let mut mock_server = MocketWrapper::new();
        mock_server.port = 10000;

        let url = server.provide(mock_server.clone(), state_gen);

        assert!(url.starts_with("https://gitlab.com/oauth/authorize"));
        assert!(url.contains("&redirect_uri=http://127.0.0.1:10000/capture/"))
    }

    #[test]
    #[serial(accesses_token)]
    fn includes_the_generated_state_in_the_url() {
        let state_gen = create_state_generator(|| STATE);
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        let url = server.provide(mock_server.clone(), state_gen);

        assert!(url.contains(format!("&state={}", STATE).as_str()))
    }

    #[test]
    #[serial(accesses_token)]
    fn uses_the_state_from_the_webserver_when_saving_token() {
        let mismatched_state = STATE + 1;
        let state_gen = create_state_generator(move || mismatched_state);
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        server.provide(mock_server.clone(), state_gen);

        assert_eq!(TokenError::NoTokenPresent, get_token().unwrap_err());
    }*/
}
