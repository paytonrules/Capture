use super::storage::Storage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InboxError {
    #[error("Unable to save todo item {0}")]
    CouldNotSaveTodo(String),

    #[error("Error loading inbox")]
    FailedToLoad(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct Todo<T: Storage> {
    storage: T,
    pub inbox: String,
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

    pub fn load(storage: T) -> Result<Self, InboxError> {
        let inbox = storage
            .load()
            .map_err(|err| InboxError::FailedToLoad(err))?;
        Ok(Todo {
            storage,
            inbox: inbox.trim().to_string(),
        })
    }

    pub fn save(&mut self, note: &String) -> Result<(), InboxError> {
        self.inbox.push_str(format!("\n- {}", note).as_str());
        self.storage
            .update(&self.inbox)
            .map_err(|err| InboxError::CouldNotSaveTodo(format!("{}", err.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbox::mock_storage::{MockError, MockStorage};
    use std::rc::Rc;

    #[test]
    fn save_note_appends_to_empty_todo_list() -> Result<(), InboxError> {
        let storage = MockStorage::new().as_rc();
        let mut todo = Todo::new(Rc::clone(&storage));

        todo.save(&"note".to_string())?;

        assert_eq!("\n- note".to_string(), storage.inbox());
        Ok(())
    }

    #[test]
    fn save_note_adds_a_newline_between_notes() -> Result<(), InboxError> {
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
    fn when_todo_is_loaded_get_inbox_from_storage() -> Result<(), InboxError> {
        let storage = MockStorage::new().with_inbox("- First todo").as_rc();

        let mut todo = Todo::load(Rc::clone(&storage))?;
        todo.save(&"second todo".to_string())?;

        assert_eq!("- First todo\n- second todo", storage.inbox());

        Ok(())
    }

    #[test]
    fn when_todo_is_loaded_trim_excess_newlines() -> Result<(), InboxError> {
        let storage = MockStorage::new().with_inbox("\n- First todo\n").as_rc();

        let mut todo = Todo::load(Rc::clone(&storage))?;
        todo.save(&"second todo".to_string())?;

        assert_eq!("- First todo\n- second todo", storage.inbox());

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
            Err(err) => assert_eq!("FailedToLoad(Test Failed To Load)", format!("{:?}", err)),
        }
    }
}
