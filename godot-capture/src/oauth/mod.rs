mod login_site;
mod provider;
mod webserver;
use lazy_static::lazy_static;
pub use provider::OAuthProvider;
use std::sync::Mutex;
use thiserror::Error;
pub use webserver::RocketWebServer;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Unable to get token lock {0}")]
    FailedToLockToken(String),

    #[error("No token present")]
    NoTokenPresent,
}

lazy_static! {
    static ref TOKEN: Mutex<Option<String>> = Mutex::new(None);
}

pub fn save_token(token: String) {
    *TOKEN.lock().unwrap() = Some(token);
}

pub fn get_token() -> Result<String, TokenError> {
    TOKEN
        .try_lock()
        .map_err(|err| TokenError::FailedToLockToken(err.to_string()))?
        .clone()
        .ok_or(TokenError::NoTokenPresent)
}

pub fn clear_token() {
    *TOKEN.lock().unwrap() = None;
}
