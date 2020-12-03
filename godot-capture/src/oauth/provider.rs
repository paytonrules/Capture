use super::save_token;
use super::webserver::WebServer;
use std::sync::mpsc::sync_channel;
use std::thread;

pub struct OAuthProvider;

impl OAuthProvider {
    pub fn new() -> Self {
        OAuthProvider
    }

    pub fn provide<T: WebServer + Send + Sync + 'static>(&self, server: T) -> String {
        let (send, token) = sync_channel(1);

        let server = server.token_sender(send);
        //         let login_url = "https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=paytonrules.Capture://capture/&response_type=token&state=100&scope=api".to_string();
        let login_url = format!("https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=http://127.0.0.1:{}/capture/&response_type=token&state=100&scope=api",
                       server.port());

        thread::spawn(move || {
            server.launch();
        });

        thread::spawn(move || {
            if let Ok(token) = token.recv() {
                save_token(token);
            }
        });

        login_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::{clear_token, get_token};
    use serial_test::serial;
    use std::sync::mpsc::SyncSender;

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
    #[serial]
    fn launches_webserver_on_provide() {
        clear_token();
        let server = OAuthProvider::new();
        let mock_server = MocketWrapper::new();

        let _url = server.provide(mock_server.clone());
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert_eq!("token".to_string(), get_token().unwrap());
    }

    #[test]
    fn returns_url_for_login_on_provide_with_port() {
        let server = OAuthProvider::new();
        let mut mock_server = MocketWrapper::new();
        mock_server.port = 10000;

        let url = server.provide(mock_server.clone());

        assert!(url.starts_with("https://gitlab.com/oauth/authorize"));
        assert!(url.contains("&redirect_uri=http://127.0.0.1:10000/capture/"))
    }
}
