use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum TodoError {
    #[error("Unable to save todo item {0}")]
    CouldNotSaveTodo(String),
}

struct Todo<'a, T: Storage> {
    storage: &'a T,
    inbox: String,
}

trait Storage {
    fn update(&self, inbox: &String) -> Result<(), Box<dyn std::error::Error>>;
    fn load(&self) -> Result<String, Box<dyn std::error::Error>>;
}

impl<'a, T> Todo<'a, T>
where
    T: Storage,
{
    fn new(storage: &'a T) -> Self {
        Todo {
            storage,
            inbox: "".to_string(),
        }
    }

    fn load(storage: &'a T) -> Result<Self, TodoError> {
        let inbox = storage.load().expect("doomed");
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

    #[derive(Debug, PartialEq, Error)]
    pub enum MockStorageError {
        #[error("{0}")]
        CantSave(String),
    }

    struct MockStorage {
        inbox: RefCell<String>,
        update_error: Option<String>,
    }

    impl MockStorage {
        fn new() -> Self {
            MockStorage {
                inbox: RefCell::new("".to_string()),
                update_error: None,
            }
        }

        fn inbox(&self) -> String {
            self.inbox.borrow().to_string()
        }

        fn set_update_to_error(&mut self, error: &str) {
            self.update_error = Some(error.to_string());
        }

        fn set_inbox(&mut self, inbox: &str) {
            *self.inbox.borrow_mut() = inbox.to_string();
        }
    }

    impl Storage for MockStorage {
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
            Ok(self.inbox.borrow().to_string())
        }
    }

    #[test]
    fn save_note_appends_to_empty_todo_list() -> Result<(), TodoError> {
        let storage = MockStorage::new();
        let mut todo = Todo::new(&storage);

        todo.save(&"note".to_string())?;

        assert_eq!("\n- note".to_string(), storage.inbox());
        Ok(())
    }

    #[test]
    fn save_note_adds_a_newline_between_notes() -> Result<(), TodoError> {
        let storage = MockStorage::new();
        let mut todo = Todo::new(&storage);

        todo.save(&"note 1".to_string())?;
        todo.save(&"note 2".to_string())?;

        assert_eq!("\n- note 1\n- note 2".to_string(), storage.inbox());
        Ok(())
    }

    #[test]
    fn when_storage_update_fails_pass_along_error() {
        let mut storage = MockStorage::new();
        storage.set_update_to_error("commit failed");
        let mut todo = Todo::new(&storage);

        let result = todo.save(&"whatever you do, don't forget".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn when_todo_is_loaded_get_inbox_from_storage() -> Result<(), TodoError> {
        let mut storage = MockStorage::new();
        storage.set_inbox("\n- First todo");

        let mut todo = Todo::load(&storage)?;
        todo.save(&"second todo".to_string())?;

        assert_eq!("\n- First todo\n- second todo", storage.inbox());

        Ok(())
    }
}
