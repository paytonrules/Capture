use super::webserver::WebServer;
use super::{TokenError, TokenReceiver};

pub struct OAuthProvider;

impl OAuthProvider {
    pub fn new() -> Self {
        OAuthProvider
    }

    pub fn provide<T, U>(&self, server: T, token_receiver: U) -> Result<String, TokenError>
    where
        T: WebServer,
        U: TokenReceiver + 'static + Send,
    {
        let state = token_receiver.state().ok_or(TokenError::NoStatePresent)?;
        let login_url = format!("https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=http://127.0.0.1:{}/capture/&response_type=token&state={}&scope=api",
                       server.port(), state);

        server.launch(move |returned_token, returned_state| {
            token_receiver.token_received(returned_token, returned_state)
        });
        /*
        thread::spawn(move || {
            while let Ok((token, returned_state)) = token.recv() {
                match token_receiver.token_received(token.as_str(), returned_state) {
                    Ok(_) => break,
                    Err(err) => eprintln!("Error receiving token {:?}", err),
                }
            }
        });*/

        Ok(login_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::oauth::token::tests::MockTokenReceiver;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;

    const STATE: i16 = 1;
    type CallbackFn = dyn FnOnce(&str, i16) -> Result<(), TokenError> + 'static;

    struct MockWebServer {
        callback: RefCell<Option<Box<CallbackFn>>>,
        port: u16,
    }

    impl MockWebServer {
        fn new() -> Rc<Self> {
            MockWebServer::new_with_port(0)
        }

        fn new_with_port(port: u16) -> Rc<Self> {
            Rc::new(MockWebServer {
                port,
                callback: RefCell::new(None),
            })
        }

        fn launched(&self) -> bool {
            self.callback.borrow().is_some()
        }

        #[allow(unused_must_use)]
        fn fire_launch_callback(&self, token: &str, state: i16) {
            let callback = self.callback.borrow_mut().take();
            callback.unwrap()(token, state);
        }
    }

    impl WebServer for Rc<MockWebServer> {
        fn launch(self, callback: impl FnOnce(&str, i16) -> Result<(), TokenError> + 'static) {
            self.callback.replace(Some(Box::new(callback)));
        }

        fn port(&self) -> u16 {
            self.port
        }
    }

    #[test]
    fn launches_webserver_on_provide() {
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(STATE));
        let mock_server = MockWebServer::new();
        let oauth_provider = OAuthProvider::new();

        let _url = oauth_provider.provide(Rc::clone(&mock_server), Arc::clone(&token_receiver));

        assert_eq!(true, mock_server.launched());
    }

    #[test]
    fn returns_url_for_login_on_provide_with_port() -> Result<(), TokenError> {
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(STATE));
        let server = OAuthProvider::new();
        let mock_server = MockWebServer::new_with_port(10000);

        let url = server.provide(Rc::clone(&mock_server), Arc::clone(&token_receiver))?;

        assert!(url.starts_with("https://gitlab.com/oauth/authorize"));
        assert!(url.contains("&redirect_uri=http://127.0.0.1:10000/capture/"));
        Ok(())
    }

    #[test]
    fn includes_the_generated_state_in_the_url() -> Result<(), TokenError> {
        let state = 894;
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(state));
        let server = OAuthProvider::new();
        let mock_server = MockWebServer::new();

        let url = server.provide(Rc::clone(&mock_server), Arc::clone(&token_receiver))?;

        assert!(url.contains("&state=894"));
        Ok(())
    }

    #[test]
    fn passes_the_returned_token_and_state_to_the_receiver() {
        let state = STATE;
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(state));
        let mock_server = MockWebServer::new();
        let oauth_provider = OAuthProvider::new();

        let _url = oauth_provider.provide(Rc::clone(&mock_server), Arc::clone(&token_receiver));

        mock_server.fire_launch_callback("token", state);

        assert_eq!(Some("token".to_string()), token_receiver.received_token());
        assert_eq!(Some(state), token_receiver.state());
    }

    #[test]
    fn does_not_provide_the_url_when_the_state_is_not_present() {
        let token_receiver = Arc::new(MockTokenReceiver::no_state_present());
        let server = OAuthProvider::new();
        let mock_server = MockWebServer::new();

        let url = server.provide(Rc::clone(&mock_server), Arc::clone(&token_receiver));

        assert_eq!(Err(TokenError::NoStatePresent), url);
    }

    #[test]
    fn uses_the_state_from_the_webserver_when_saving_token() -> Result<(), TokenError> {
        let state_from_url = STATE;
        let mismatched_state = STATE + 1;
        let token_receiver = Arc::new(MockTokenReceiver::new_with_state(mismatched_state));
        let server = OAuthProvider::new();
        let mock_server = MockWebServer::new();

        server.provide(Rc::clone(&mock_server), Arc::clone(&token_receiver))?;
        mock_server.fire_launch_callback("irrelevant", state_from_url);

        assert_eq!(Some(state_from_url), token_receiver.received_state());
        Ok(())
    }
}
