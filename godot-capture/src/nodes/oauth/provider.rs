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
        T: WebServer,
        U: TokenReceiver + Send + Clone + 'static,
    {
        let (send, token) = sync_channel(1);

        let server = server.token_sender(send);
        let state = token_receiver.state().ok_or(TokenError::NoStatePresent)?;
        let login_url = format!("https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=http://127.0.0.1:{}/capture/&response_type=token&state={}&scope=api",
                       server.port(), state);

        let cloned_tr = token_receiver.clone();
        server.launch(move |returned_token, returned_state| {
            cloned_tr.token_received(returned_token, returned_state);
        });

        thread::spawn(move || {
            while let Ok((token, returned_state)) = token.recv() {
                match token_receiver.token_received(token.as_str(), returned_state) {
                    Ok(_) => break,
                    Err(err) => eprintln!("Error receiving token {:?}", err),
                }
            }
        });

        Ok(login_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::oauth::token::tests::MockTokenReceiver;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::mpsc::SyncSender;
    use std::sync::Arc;

    const STATE: i16 = 1;
    type CallbackFn = FnOnce(&str, i16) -> () + 'static;

    struct MockWebServer {
        sync_sender: RefCell<Option<SyncSender<(String, i16)>>>,
        launched: RefCell<bool>,
        callback: RefCell<Option<Box<CallbackFn>>>,
        port: u16,
    }

    impl MockWebServer {
        fn new() -> Self {
            MockWebServer::new_with_port(0)
        }

        fn new_with_port(port: u16) -> Self {
            MockWebServer {
                sync_sender: RefCell::new(None),
                port,
                callback: RefCell::new(None),
                launched: RefCell::new(false),
            }
        }

        fn launched(&self) -> bool {
            *self.launched.borrow()
        }

        fn fire_launch_callback(&self, token: &str, state: i16) {
            let callback = self.callback.borrow_mut().take();
            callback.unwrap()("bad", 1);
        }

        fn send_token_and_state(&self, token: &str, state: i16) {
            assert!(self.sync_sender.borrow().is_some());
            self.sync_sender
                .borrow()
                .as_ref()
                .map(|sender| sender.send((token.to_string(), state)));
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    impl WebServer for Rc<MockWebServer> {
        fn token_sender(self, sender: SyncSender<(String, i16)>) -> Self {
            self.sync_sender.replace(Some(sender));
            self
        }

        fn launch(self, callback: impl FnOnce(&str, i16) -> () + 'static) {
            self.launched.replace(true);
            self.callback.replace(Some(Box::new(callback)));
        }

        fn port(&self) -> u16 {
            self.port
        }
    }

    #[test]
    fn launches_webserver_on_provide() {
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(1));
        let mock_server = Rc::new(MockWebServer::new());
        let oauth_provider = OAuthProvider::new();

        let _url = oauth_provider.provide(Rc::clone(&mock_server), Arc::clone(&token_receiver));

        assert_eq!(true, mock_server.launched());
    }
    /*
    #[test]
    fn returns_url_for_login_on_provide_with_port() -> Result<(), TokenError> {
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(1));
        let server = OAuthProvider::new();
        let mock_server = Arc::new(MockWebServer::new_with_port(10000));

        let url = server.provide(Arc::clone(&mock_server), Arc::clone(&token_receiver))?;

        assert!(url.starts_with("https://gitlab.com/oauth/authorize"));
        assert!(url.contains("&redirect_uri=http://127.0.0.1:10000/capture/"));
        Ok(())
    }

    #[test]
    fn captures_shutdown_hook_on_launch_token_receiver() {
        let state = STATE;
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(state));
        let mock_server = Arc::new(MockWebServer::new());
        let oauth_provider = OAuthProvider::new();

        let _url = oauth_provider.provide(Arc::clone(&mock_server), Arc::clone(&token_receiver));

        mock_server.fire_launch_callback("token", state);

        assert_eq!(Some("token".to_string()), token_receiver.received_token());
        assert_eq!(Some(state), token_receiver.state());
    }

    #[test]
    fn includes_the_generated_state_in_the_url() -> Result<(), TokenError> {
        let state = 894;
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(state));
        let server = OAuthProvider::new();
        let mock_server = Arc::new(MockWebServer::new());

        let url = server.provide(Arc::clone(&mock_server), Arc::clone(&token_receiver))?;

        assert!(url.contains("&state=894"));
        Ok(())
    }

    #[test]
    fn does_not_provide_the_url_when_the_state_is_not_present() {
        let token_receiver = Arc::new(MockTokenReceiver::no_state_present());
        let server = OAuthProvider::new();
        let mock_server = Arc::new(MockWebServer::new());

        let url = server.provide(Arc::clone(&mock_server), Arc::clone(&token_receiver));

        assert_eq!(Err(TokenError::NoStatePresent), url);
    }

    #[test]
    fn uses_the_state_from_the_webserver_when_saving_token() -> Result<(), TokenError> {
        let state_from_url = STATE;
        let mismatched_state = STATE + 1;
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(mismatched_state));
        let server = OAuthProvider::new();
        let mock_server = Arc::new(MockWebServer::new());

        server.provide(Arc::clone(&mock_server), Arc::clone(&token_receiver))?;
        mock_server.send_token_and_state("irrelevant", state_from_url);

        assert_eq!(Some(state_from_url), token_receiver.received_state());
        Ok(())
    }

    #[test]
    fn when_state_doesnt_match_keep_listening() -> Result<(), TokenError> {
        let desired_state = STATE;
        let mismatched_state = STATE + 1;
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(desired_state));
        let server = OAuthProvider::new();
        let mock_server = Arc::new(MockWebServer::new());

        server.provide(Arc::clone(&mock_server), Arc::clone(&token_receiver))?;

        mock_server.send_token_and_state("token", mismatched_state);
        assert_eq!(None, token_receiver.received_token());

        mock_server.send_token_and_state("token", desired_state);
        assert_eq!(Some("token".to_string()), token_receiver.received_token());
        Ok(())
    }*/
}
