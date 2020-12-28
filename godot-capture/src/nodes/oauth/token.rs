use lazy_static::lazy_static;
use rand::random;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TokenError {
    #[error("OAuth state param doesn't match")]
    StateDoesntMatch,

    #[error("Can only authenticate once")]
    AlreadyAuthenticated,

    #[error("No state value is present, are you already authenticated?")]
    NoStatePresent,
}

pub trait TokenReceiver {
    fn state(&self) -> Option<i16>;
    fn token_received(&self, token: &str, state: i16) -> Result<(), TokenError>;
}

pub trait TokenRetriever {
    fn token(&self) -> Option<String>;
}

#[derive(PartialEq, Debug, Clone)]
pub enum AuthMachine {
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

lazy_static! {
    static ref MACHINE: Mutex<AuthMachine> = Mutex::new(AuthMachine::new(random));
}

pub struct AuthState;

impl AuthState {
    pub fn initialize(machine: AuthMachine) {
        *MACHINE.lock().unwrap() = machine;
    }

    pub fn get() -> AuthState {
        AuthState
    }
}

impl TokenRetriever for AuthState {
    fn token(&self) -> Option<String> {
        MACHINE.lock().unwrap().token()
    }
}

impl TokenReceiver for AuthState {
    fn state(&self) -> Option<i16> {
        MACHINE.lock().unwrap().state()
    }

    fn token_received(&self, token: &str, state: i16) -> Result<(), TokenError> {
        let auth_machine = MACHINE.lock().unwrap().clone();
        let updated_auth_machine = auth_machine.token_received(token, state)?;

        *MACHINE.lock().unwrap() = updated_auth_machine;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serial_test::serial;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};

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

    impl AuthState {
        fn new(machine: AuthMachine) -> Self {
            AuthState::initialize(machine);
            AuthState
        }
    }

    #[test]
    #[serial(using_auth_state)]
    fn new_auth_state_starts_with_state() {
        let auth_state = AuthState::new(AuthMachine::new(|| 20));

        assert_eq!(Some(20), auth_state.state());
    }

    #[test]
    #[serial(using_auth_state)]
    fn retrieve_existing_auth_state() {
        let _auth_state = AuthState::new(AuthMachine::new(|| 40));
        let auth_state = AuthState::get();

        assert_eq!(Some(40), auth_state.state());
    }

    #[test]
    #[serial(using_auth_state)]
    fn valid_token_recieved_saves_the_token() -> Result<(), TokenError> {
        let _auth_state = AuthState::new(AuthMachine::new(|| 100));

        AuthState::get().token_received("THE TOKEN", 100)?;

        assert_eq!(Some("THE TOKEN".to_string()), AuthState::get().token());
        Ok(())
    }

    pub struct MockTokenReceiver {
        state: Option<i16>,
        received_token: Mutex<RefCell<Option<String>>>,
        received_state: Mutex<RefCell<Option<i16>>>,
    }

    impl MockTokenReceiver {
        pub fn new(state: i16) -> Self {
            MockTokenReceiver {
                state: Some(state),
                received_token: Mutex::new(RefCell::new(None)),
                received_state: Mutex::new(RefCell::new(None)),
            }
        }

        pub fn no_state_present() -> Self {
            MockTokenReceiver {
                state: None,
                received_token: Mutex::new(RefCell::new(None)),
                received_state: Mutex::new(RefCell::new(None)),
            }
        }

        pub fn received_token(&self) -> Option<String> {
            (*self.received_token.lock().unwrap().borrow()).clone()
        }

        pub fn received_state(&self) -> Option<i16> {
            (*self.received_state.lock().unwrap().borrow()).clone()
        }
    }

    impl TokenReceiver for Arc<MockTokenReceiver> {
        fn state(&self) -> Option<i16> {
            self.state
        }

        fn token_received(&self, token: &str, state: i16) -> Result<(), TokenError> {
            *self.received_state.lock().unwrap().borrow_mut() = Some(state);
            if self.state == Some(state) {
                *self.received_token.lock().unwrap().borrow_mut() = Some(token.to_string());
                Ok(())
            } else {
                Err(TokenError::StateDoesntMatch)
            }
        }
    }

    impl TokenReceiver for Rc<MockTokenReceiver> {
        fn state(&self) -> Option<i16> {
            self.state
        }

        fn token_received(&self, token: &str, state: i16) -> Result<(), TokenError> {
            *self.received_state.lock().unwrap().borrow_mut() = Some(state);
            if self.state == Some(state) {
                *self.received_token.lock().unwrap().borrow_mut() = Some(token.to_string());
                Ok(())
            } else {
                Err(TokenError::StateDoesntMatch)
            }
        }
    }
}
