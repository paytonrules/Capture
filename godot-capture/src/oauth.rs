use gdnative::prelude::*;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

struct OAuthServer {
    address: String,
    token: Arc<Mutex<Option<String>>>,
}

impl OAuthServer {
    fn new(address: String) -> Self {
        OAuthServer {
            address,
            token: Arc::new(Mutex::new(None)),
        }
    }

    fn start(&self) {
        let address = self.address.clone();
        let token = self.token.clone();
        thread::spawn(move || {
            let listener = TcpListener::bind(address).unwrap();

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = [0; 512];
                        stream.read(&mut buffer).unwrap();

                        let mut buffer_slice = buffer.split(|c| *c == b' ');
                        let _method = buffer_slice.next().unwrap();
                        let url = buffer_slice.next().unwrap();
                        let url_string = std::str::from_utf8(url).unwrap();

                        let mut url_slice = url_string.split('#');
                        let _route = url_slice.next().unwrap();
                        let param_string = url_slice.next().unwrap();

                        let mut param_slice = param_string.split('&');
                        if let Some(token_pair) =
                            param_slice.find(|pair| pair.starts_with("access_token="))
                        {
                            let received_token = token_pair.split('=').last();
                            token
                                .lock()
                                .unwrap()
                                .replace(received_token.map(|s| s.to_string()).unwrap());
                        }
                    }
                    Err(_err) => println!("Error of some kind {}", _err),
                }
            }
        });
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Listener;

#[methods]
impl Listener {
    fn new(_owner: &Node) -> Self {
        Listener
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_stores_access_token_from_redirect() -> std::io::Result<()> {
        let server = OAuthServer::new("127.0.0.1:8080".to_string());
        server.start();
        let token = server.token.clone();

        let redirect = b"GET /capture#access_token=token&state=state&token_type=bearer&expires_in=3600 HTTP/1.1\r\n";
        let mut connection = TcpStream::connect("127.0.0.1:8080")?;
        connection.write(redirect);

        sleep(Duration::new(3, 0));
        assert_eq!(token.lock().unwrap().as_ref(), Some(&"token".to_string()));

        Ok(())
    }

    // test shutdown
    // test state matches
    // Whatcha gonna do with expiration?
    // Invalid URLs
    // Ensure we hit a valid port
}
