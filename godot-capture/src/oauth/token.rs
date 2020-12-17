use lazy_static::lazy_static;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TokenError {
    #[error("Unable to get token lock {0}")]
    FailedToLockToken(String),

    #[error("No token present")]
    NoTokenPresent,

    #[error("Oauth state param doesn't match")]
    StateDoesntMatch,
}

lazy_static! {
    static ref TOKEN: Mutex<Option<String>> = Mutex::new(None);
    static ref STATE: Mutex<Option<i16>> = Mutex::new(None);
}

pub fn create_state_generator(rand: impl Fn() -> i16 + 'static) -> Box<dyn Fn() -> i16> {
    Box::new(move || {
        let val = rand();
        *STATE.lock().unwrap() = Some(val);
        val
    })
}

pub fn save_token(token: String, state: i16) -> Result<(), TokenError> {
    match *STATE.lock().unwrap() {
        Some(actual_state) if actual_state == state => {
            *TOKEN.lock().unwrap() = Some(token);
            Ok(())
        }
        _ => Err(TokenError::StateDoesntMatch),
    }
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
    use serial_test::serial;

    #[test]
    #[serial(accesses_token)]
    fn get_token_errors_with_no_token() {
        clear_token();

        let result = get_token();

        match result {
            Ok(_) => assert!(false, "Expected a failure, got an Ok"),
            Err(err) => assert_eq!(err, TokenError::NoTokenPresent),
        }
    }

    #[test]
    #[serial(accesses_token)]
    fn get_token_returns_the_string_when_present() -> Result<(), TokenError> {
        clear_token();
        create_state_generator(|| 0)();
        save_token("TOKEN".to_string(), 0);

        let token = get_token()?;

        assert_eq!(token, "TOKEN");
        Ok(())
    }

    #[test]
    #[serial(accesses_token)]
    fn save_token_fails_when_the_state_doesnt_match() {
        create_state_generator(|| 900)();

        assert_eq!(
            TokenError::StateDoesntMatch,
            save_token("TOKEN".to_string(), 1000).unwrap_err()
        );
    }

    #[test]
    #[serial(accesses_token)]
    fn state_generator_returns_the_generated_val() {
        let val = create_state_generator(|| 900)();

        assert_eq!(val, 900);
    }
}
