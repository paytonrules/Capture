use crate::todo::{GitlabStorage, Storage, Todo, TodoError};
use gdnative::api::{AcceptDialog, TextEdit, TextureButton};
use gdnative::prelude::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use thiserror::Error;

lazy_static! {
    static ref TOKEN: Mutex<Option<String>> = Mutex::new(None);
}

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Unable to get token lock {0}")]
    FailedToLockToken(String),

    #[error("No token present")]
    NoTokenPresent,

    #[error("Error getting todo list: {0}")]
    ErrorGettingTodoList(#[from] TodoError),
}

#[derive(NativeClass)]
#[inherit(TextureButton)]
pub struct Remember {
    todo: Option<Todo<GitlabStorage>>,
}

#[methods]
impl Remember {
    fn new(_owner: &TextureButton) -> Self {
        Remember { todo: None }
    }

    #[export]
    fn _ready(&mut self, owner: TRef<TextureButton>) {
        self.todo = create_storage()
            .and_then(|storage| load_todos(storage))
            .ok();

        match &self.todo {
            Some(todos) => update_view(owner, &todos.inbox),
            None => godot_error!("Nothing to do"),
        }
    }

    #[export]
    fn _save_me(&mut self, owner: TRef<TextureButton>) {
        let new_todo = new_todo_window(owner);

        if let Some(todos) = &mut self.todo {
            match save_new_todo(todos, &new_todo.text().to_string()) {
                Ok(_) => update_view(owner, &todos.inbox),
                Err(err) => {
                    let dialog = AcceptDialog::new();
                    dialog.set_text(format!("Could Not Save Todo: {:?}", err));
                    let dialog = unsafe { dialog.assume_shared() };
                    owner.add_child(dialog, false);
                    let dialog = unsafe { dialog.assume_safe() };
                    dialog.popup_centered(Vector2::new(0.0, 0.0));
                }
            }
        }
    }
}

fn update_view(owner: TRef<TextureButton>, inbox: &str) {
    let inbox_view = owner
        .get_node("/root/CaptureNote/CenterContainer/VBoxContainer/Recent Todos")
        .map(|node| unsafe { node.assume_safe() })
        .and_then(|node| node.cast::<Label>())
        .expect("Recent Todos node is missing");
    inbox_view.set_text(truncate_to_latest_todos(inbox));
    let new_todo_window = new_todo_window(owner);
    new_todo_window.set_text("");
}

fn new_todo_window(owner: TRef<TextureButton>) -> TRef<TextEdit> {
    owner
        .get_node("/root/CaptureNote/CenterContainer/VBoxContainer/New Todo")
        .map(|node| unsafe { node.assume_safe() })
        .and_then(|node| node.cast::<TextEdit>())
        .expect("New Todo node is missing")
}

pub fn save_token(token: String) {
    *TOKEN.lock().unwrap() = Some(token);
}

fn truncate_to_latest_todos(inbox: &str) -> String {
    let todos = inbox
        .split('\n')
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
    let todo_count = todos.len();

    if todo_count > 4 {
        todos
            .iter()
            .skip(todo_count - 4)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join("\n")
            .to_string()
    } else {
        inbox.to_string()
    }
}

fn create_storage() -> Result<GitlabStorage, CaptureError> {
    let token = TOKEN
        .try_lock()
        .map_err(|err| CaptureError::FailedToLockToken(err.to_string()))?
        .clone();

    let token = token.ok_or(CaptureError::NoTokenPresent)?;

    Ok(GitlabStorage::new(token.to_string()))
}

fn load_todos<T: Storage>(storage: T) -> Result<Todo<T>, CaptureError> {
    Todo::load(storage).map_err(|err| CaptureError::ErrorGettingTodoList(err))
}

fn save_new_todo<T: Storage>(todos: &mut Todo<T>, new_todo: &str) -> Result<(), CaptureError> {
    todos.save(&new_todo.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::{MockError, MockStorage};
    use serial_test::serial;
    use std::rc::Rc;

    #[test]
    fn take_all_items_up_to_four() {
        let full_list = "- one
- two
- three
- four"
            .to_string();
        assert_eq!(full_list, truncate_to_latest_todos(&full_list));
    }

    #[test]
    fn take_only_the_last_four() {
        let full_list = "- skip
- one
- two
- three
- four"
            .to_string();
        let expected = "- one
- two
- three
- four"
            .to_string();

        assert_eq!(expected, truncate_to_latest_todos(&full_list));
    }

    #[test]
    fn take_all_the_entries_if_there_are_lt_four() {
        let full_list = "- one
- two
- three"
            .to_string();
        let expected = "- one
- two
- three"
            .to_string();

        assert_eq!(expected, truncate_to_latest_todos(&full_list));
    }

    #[test]
    #[serial]
    fn when_a_token_is_present_create_storage() -> Result<(), Box<dyn std::error::Error>> {
        save_token("token".to_string());

        let storage = create_storage()?;

        assert_eq!("token", storage.token);
        Ok(())
    }

    #[test]
    #[serial]
    fn when_a_token_is_not_present() -> Result<(), Box<dyn std::error::Error>> {
        *TOKEN.lock().unwrap() = None;
        let storage = create_storage();

        match storage {
            Err(CaptureError::NoTokenPresent) => assert!(true, "correct error"),
            Ok(token) => assert!(false, println!("unexpected Ok result - {:?}", token)),
            Err(err) => assert!(false, println!("unexpected error thrown {:?}", err)),
        }

        Ok(())
    }

    #[test]
    fn load_todos_from_storage() {
        let storage = Rc::new(MockStorage::new().with_inbox("-first\nsecond"));

        let todos = load_todos(storage);
        assert!(todos.is_ok());
        assert_eq!("-first\nsecond", todos.unwrap().inbox);
    }

    #[test]
    fn map_load_todos_failure_to_capture_error() {
        let storage = Rc::new(MockStorage::new().with_load_error(MockError::TestFailedToLoad));

        let todos = load_todos(storage);
        match todos {
            Err(CaptureError::ErrorGettingTodoList(err)) => match err {
                TodoError::FailedToLoad(sub_err) => match sub_err.downcast::<MockError>() {
                    Ok(MockError::TestFailedToLoad) => assert!(true, "correct error"),
                    _ => assert!(false, "incorrect error"),
                },
                _ => assert!(false, "incorrect error"),
            },
            Ok(_) => assert!(false, println!("unexpected Ok result")),
            Err(err) => assert!(false, println!("unexpected error thrown {:?}", err)),
        }
    }

    #[test]
    fn save_new_todo_saves() -> Result<(), Box<dyn std::error::Error>> {
        let storage = Rc::new(MockStorage::new().with_inbox("- one"));
        let mut todos = Todo::load(storage)?;

        save_new_todo(&mut todos, "two")?;

        assert_eq!("- one\n- two", todos.inbox);
        Ok(())
    }
}
