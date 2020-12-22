use super::{TokenError, TokenReceiver};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

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
        *self.received_token.lock().unwrap().borrow_mut() = Some(token.to_string());
        *self.received_state.lock().unwrap().borrow_mut() = Some(state);
        Ok(())
    }
}

impl TokenReceiver for Rc<MockTokenReceiver> {
    fn state(&self) -> Option<i16> {
        self.state
    }

    fn token_received(&self, token: &str, state: i16) -> Result<(), TokenError> {
        *self.received_token.lock().unwrap().borrow_mut() = Some(token.to_string());
        *self.received_state.lock().unwrap().borrow_mut() = Some(state);
        Ok(())
    }
}
