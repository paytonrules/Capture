use lazy_static::lazy_static;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_token_errors_with_no_token() {
        clear_token();

        let result = get_token();

        match result {
            Ok(_) => assert!(false, "Expected a failure, got an Ok"),
            Err(err) => assert_eq!(err, TokenError::NoTokenPresent),
        }
    }

    #[test]
    fn get_token_returns_the_string_when_present() -> Result<(), TokenError> {
        clear_token();
        save_token("TOKEN".to_string());

        let token = get_token()?;

        assert_eq!(token, "TOKEN");
        Ok(())
    }
}
