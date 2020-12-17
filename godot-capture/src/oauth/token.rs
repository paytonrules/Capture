use lazy_static::lazy_static;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TokenError {
    #[error("Unable to get token lock {0}")]
    FailedToLockToken(String),

    #[error("No token present")]
    NoTokenPresent,

    #[error("OAuth state param doesn't match")]
    StateDoesntMatch,

    #[error("Can only authenticate once")]
    AlreadyAuthenticated,
}

lazy_static! {
    static ref TOKEN: Mutex<Option<String>> = Mutex::new(None);
    static ref STATE: Mutex<Option<i16>> = Mutex::new(None);
}

#[derive(PartialEq, Debug)]
enum AuthMachine {
    UnAuthenticated(i16),
    Authenticated(String),
}

impl AuthMachine {
    pub fn new(rand: impl Fn() -> i16 + 'static) -> AuthMachine {
        AuthMachine::UnAuthenticated(rand())
    }

    pub fn state(&self) -> Option<i16> {
        match &self {
            AuthMachine::UnAuthenticated(state) => Some(*state),
            _ => None,
        }
    }

    pub fn token_received(self, token: &str, state: i16) -> Result<AuthMachine, TokenError> {
        match self {
            AuthMachine::UnAuthenticated(actual_state) if state == actual_state => {
                Ok(AuthMachine::Authenticated(token.to_string()))
            }
            AuthMachine::UnAuthenticated(_) => Err(TokenError::StateDoesntMatch),
            AuthMachine::Authenticated(_) => Err(TokenError::AlreadyAuthenticated),
        }
    }

    pub fn token(&self) -> Option<String> {
        match &self {
            AuthMachine::Authenticated(token) => Some(token.to_string()),
            _ => None,
        }
    }
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

    #[test]
    fn starts_with_state_value() {
        let authentication = AuthMachine::new(|| 10);

        assert_eq!(Some(10), authentication.state());
        assert_eq!(None, authentication.token());
    }

    #[test]
    fn authenticates_with_a_token() -> Result<(), TokenError> {
        let authentication = AuthMachine::new(|| 20);

        let authentication = authentication.token_received("TOKEN", 20)?;

        assert_eq!(Some("TOKEN".to_string()), authentication.token());
        assert_eq!(None, authentication.state());
        Ok(())
    }

    #[test]
    fn requires_state_to_match_to_authenticate_token() {
        let authentication = AuthMachine::new(|| 20);

        let authentication = authentication.token_received("TOKEN", 10);

        assert_eq!(Err(TokenError::StateDoesntMatch), authentication);
    }

    #[test]
    fn only_authenticates_once() -> Result<(), TokenError> {
        let authentication = AuthMachine::new(|| 20);

        let authentication = authentication
            .token_received("TOKEN", 20)?
            .token_received("TOKEN", 20);

        assert_eq!(Err(TokenError::AlreadyAuthenticated), authentication);
        Ok(())
    }
}
