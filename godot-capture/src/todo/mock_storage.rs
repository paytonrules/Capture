use super::storage::Storage;
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum MockStorageError {
    #[error("{0}")]
    CantSave(String),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum MockError {
    #[error("Test Failed To Load")]
    TestFailedToLoad,
}

pub struct MockStorage {
    inbox: RefCell<String>,
    update_error: Option<String>,
    load_error: Option<MockError>,
}

impl MockStorage {
    pub fn new() -> Self {
        MockStorage {
            inbox: RefCell::new("".to_string()),
            update_error: None,
            load_error: None,
        }
    }

    pub fn inbox(&self) -> String {
        self.inbox.borrow().to_string()
    }

    pub fn with_update_error(mut self, error: &str) -> Self {
        self.update_error = Some(error.to_string());
        self
    }

    pub fn with_inbox(self, inbox: &str) -> Self {
        *self.inbox.borrow_mut() = inbox.to_string();
        self
    }

    pub fn with_load_error(mut self, error: MockError) -> Self {
        self.load_error = Some(error);
        self
    }

    pub fn as_rc(self) -> Rc<Self> {
        Rc::new(self)
    }
}

impl Storage for Rc<MockStorage> {
    fn update(&self, inbox: &String) -> anyhow::Result<()> {
        match &self.update_error {
            None => {
                *self.inbox.borrow_mut() = inbox.to_string();
                Ok(())
            }
            Some(update_error) => Err(MockStorageError::CantSave(update_error.to_string()).into()),
        }
    }

    fn load(&self) -> anyhow::Result<String> {
        match &self.load_error {
            None => Ok(self.inbox.borrow().to_string()),
            Some(err) => Err(err.clone().into()),
        }
    }
}
