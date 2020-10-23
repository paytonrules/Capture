use thiserror::Error;

#[derive(Debug, Error)]
pub enum TodoError {
    #[error("Unable to save todo item {0}")]
    CouldNotSaveTodo(String),

    #[error("Error loading inbox")]
    FailedToLoad(#[from] Box<dyn std::error::Error>),
}

pub struct Todo<T: Storage> {
    storage: T,
    inbox: String,
}

impl<T> Todo<T>
where
    T: Storage,
{
    pub fn new(storage: T) -> Self {
        Todo {
            storage,
            inbox: "".to_string(),
        }
    }

    fn load(storage: T) -> Result<Self, TodoError> {
        let inbox = storage.load().map_err(|err| TodoError::FailedToLoad(err))?;
        Ok(Todo { storage, inbox })
    }

    fn save(&mut self, note: &String) -> Result<(), TodoError> {
        self.inbox.push_str(format!("\n- {}", note).as_str());
        self.storage
            .update(&self.inbox)
            .map_err(|err| TodoError::CouldNotSaveTodo(format!("{}", err.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, PartialEq, Error)]
    pub enum MockStorageError {
        #[error("{0}")]
        CantSave(String),
    }

    #[derive(Debug, Clone, Copy, Error)]
    pub enum MockError {
        #[error("Failed To Load")]
        TestFailedToLoad,
    }

    struct MockStorage {
        inbox: RefCell<String>,
        update_error: Option<String>,
        load_error: Option<MockError>,
    }

    impl MockStorage {
        fn new() -> Self {
            MockStorage {
                inbox: RefCell::new("".to_string()),
                update_error: None,
                load_error: None,
            }
        }

        fn inbox(&self) -> String {
            self.inbox.borrow().to_string()
        }

        fn with_update_error(mut self, error: &str) -> Self {
            self.update_error = Some(error.to_string());
            self
        }

        fn with_inbox(mut self, inbox: &str) -> Self {
            *self.inbox.borrow_mut() = inbox.to_string();
            self
        }

        fn with_load_error(mut self, error: MockError) -> Self {
            self.load_error = Some(error);
            self
        }

        fn as_rc(self) -> Rc<Self> {
            Rc::new(self)
        }
    }

    impl Storage for Rc<MockStorage> {
        fn update(&self, inbox: &String) -> Result<(), Box<dyn std::error::Error>> {
            if let Some(update_error) = &self.update_error {
                Err(Box::new(MockStorageError::CantSave(
                    update_error.to_string(),
                )))
            } else {
                *self.inbox.borrow_mut() = inbox.to_string();
                Ok(())
            }
        }

        fn load(&self) -> Result<String, Box<dyn std::error::Error>> {
            match &self.load_error {
                None => Ok(self.inbox.borrow().to_string()),
                Some(err) => Err(Box::new(*err)),
            }
        }
    }
    #[test]
    fn save_note_appends_to_empty_todo_list() -> Result<(), TodoError> {
        let storage = MockStorage::new().as_rc();
        let mut todo = Todo::new(Rc::clone(&storage));

        todo.save(&"note".to_string())?;

        assert_eq!("\n- note".to_string(), storage.inbox());
        Ok(())
    }

    #[test]
    fn save_note_adds_a_newline_between_notes() -> Result<(), TodoError> {
        let storage = MockStorage::new().as_rc();
        let mut todo = Todo::new(Rc::clone(&storage));

        todo.save(&"note 1".to_string())?;
        todo.save(&"note 2".to_string())?;

        assert_eq!("\n- note 1\n- note 2".to_string(), storage.inbox());
        Ok(())
    }

    #[test]
    fn when_storage_update_fails_pass_along_error() {
        let storage = MockStorage::new()
            .with_update_error("commit failed")
            .as_rc();
        let mut todo = Todo::new(Rc::clone(&storage));

        let result = todo.save(&"whatever you do, don't forget".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn when_todo_is_loaded_get_inbox_from_storage() -> Result<(), TodoError> {
        let storage = MockStorage::new().with_inbox("\n- First todo").as_rc();

        let mut todo = Todo::load(Rc::clone(&storage))?;
        todo.save(&"second todo".to_string())?;

        assert_eq!("\n- First todo\n- second todo", storage.inbox());

        Ok(())
    }

    #[test]
    fn when_todo_list_cant_be_loaded_return_that_result() {
        let storage = MockStorage::new()
            .with_load_error(MockError::TestFailedToLoad)
            .as_rc();

        let todo = Todo::load(Rc::clone(&storage));
        match todo {
            Ok(_) => assert!(false, "Test Failed: Expected load to fail, it succeeded"),
            Err(err) => assert_eq!("FailedToLoad(TestFailedToLoad)", format!("{:?}", err)),
        }
    }
}
