use super::storage::Storage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InboxError {
    #[error("Unable to save reminder {0}")]
    CouldNotSaveReminder(String),

    #[error("Error loading inbox")]
    FailedToLoad(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct Inbox<T: Storage> {
    storage: T,
    reminders: Vec<String>,
}

impl<T> Inbox<T>
where
    T: Storage,
{
    fn new(storage: T) -> Self {
        Inbox {
            storage,
            reminders: Vec::new(),
        }
    }

    pub fn load(storage: T) -> Result<Self, InboxError> {
        let raw_inbox = storage
            .load()
            .map_err(|err| InboxError::FailedToLoad(err))?;
        let mut inbox = Inbox::new(storage);
        inbox.reminders = raw_inbox
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.eq(&"* Inbox"))
            .filter(|line| !line.is_empty())
            .map(|reminder| reminder.strip_prefix("** ").unwrap_or(reminder))
            .map(|reminder| String::from(reminder))
            .collect();
        Ok(inbox)
    }

    pub fn save(&mut self, note: &str) -> Result<(), InboxError> {
        self.reminders.push(note.into());
        let mut reminder_string = String::from("* Inbox");
        for reminder in self.reminders.iter() {
            reminder_string.push_str(format!("\n** {}", reminder).as_str());
        }
        self.storage
            .update(&reminder_string)
            .map_err(|err| InboxError::CouldNotSaveReminder(format!("{}", err.to_string())))
    }

    pub fn reminders(&self) -> String {
        self.reminders.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::inbox::storage::tests::{MockError, MockStorage};
    use std::rc::Rc;

    #[test]
    fn save_note_appends_to_empty_todo_list() -> Result<(), InboxError> {
        let storage = MockStorage::new().as_rc();
        let mut todo = Inbox::new(Rc::clone(&storage));

        todo.save(&"note".to_string())?;

        assert_eq!("* Inbox\n** note".to_string(), storage.inbox());
        Ok(())
    }

    #[test]
    fn save_note_adds_a_newline_between_notes() -> Result<(), InboxError> {
        let storage = MockStorage::new().as_rc();
        let mut todo = Inbox::new(Rc::clone(&storage));

        todo.save(&"note 1".to_string())?;
        todo.save(&"note 2".to_string())?;

        assert_eq!("* Inbox\n** note 1\n** note 2".to_string(), storage.inbox());
        Ok(())
    }

    #[test]
    fn when_storage_update_fails_pass_along_error() {
        let storage = MockStorage::new()
            .with_update_error("commit failed")
            .as_rc();
        let mut todo = Inbox::new(Rc::clone(&storage));

        let result = todo.save(&"whatever you do, don't forget".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn when_todo_is_loaded_get_inbox_from_storage() -> Result<(), InboxError> {
        let storage = MockStorage::new()
            .with_inbox("* Inbox\n** First todo")
            .as_rc();

        let mut todo = Inbox::load(Rc::clone(&storage))?;
        todo.save(&"second todo".to_string())?;

        assert_eq!("* Inbox\n** First todo\n** second todo", storage.inbox());

        Ok(())
    }

    #[test]
    fn when_todo_is_loaded_trim_excess_newlines() -> Result<(), InboxError> {
        let storage = MockStorage::new().with_inbox("\n** First todo\n").as_rc();

        let mut todo = Inbox::load(Rc::clone(&storage))?;
        todo.save(&"second todo".to_string())?;

        assert_eq!("* Inbox\n** First todo\n** second todo", storage.inbox());

        Ok(())
    }

    #[test]
    fn when_todo_is_loaded_trim_excess_whitespace() -> Result<(), InboxError> {
        let storage = MockStorage::new()
            .with_inbox("* Inbox\n  ** First todo   \n")
            .as_rc();

        let mut todo = Inbox::load(Rc::clone(&storage))?;
        todo.save(&"second todo".to_string())?;

        assert_eq!("* Inbox\n** First todo\n** second todo", storage.inbox());

        Ok(())
    }

    #[test]
    fn when_a_reminder_has_invalid_format_just_keep_it() -> Result<(), InboxError> {
        let storage = MockStorage::new()
            .with_inbox("* Inbox\n- First todo\n")
            .as_rc();

        let mut todo = Inbox::load(Rc::clone(&storage))?;
        todo.save(&"second todo".to_string())?;

        assert_eq!("* Inbox\n** - First todo\n** second todo", storage.inbox());

        Ok(())
    }

    #[test]
    fn when_todo_list_cant_be_loaded_return_that_result() {
        let storage = MockStorage::new()
            .with_load_error(MockError::TestFailedToLoad)
            .as_rc();

        let todo = Inbox::load(Rc::clone(&storage));
        match todo {
            Ok(_) => assert!(false, "Test Failed: Expected load to fail, it succeeded"),
            Err(err) => assert_eq!("FailedToLoad(Test Failed To Load)", format!("{:?}", err)),
        }
    }
}
