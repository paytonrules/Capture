use crate::todo::TodoError;
use crate::todo::{GitlabStorage, Todo};
use gdnative::api::TextureButton;
use gdnative::prelude::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use thiserror::Error;

lazy_static! {
    static ref TOKEN: Mutex<Option<String>> = Mutex::new(None);
}

pub fn save_token(token: String) {
    *TOKEN.lock().unwrap() = Some(token);
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
        match self.load_todos() {
            Ok(todos) => {
                let label = owner
                    .get_node("/root/CaptureNote/CenterContainer/VBoxContainer/Recent Todos")
                    .map(|node| unsafe { node.assume_safe() })
                    .and_then(|node| node.cast::<Label>())
                    .expect("Recent Todos node is missing");
                label.set_text(get_latest_todos(&todos.inbox));
                self.todo = Some(todos);
            }
            Err(err) => godot_error!("Error! {:?}", err),
        }
    }

    fn load_todos(&self) -> Result<Todo<GitlabStorage>, CaptureError> {
        let token = TOKEN
            .try_lock()
            .map_err(|err| CaptureError::FailedToLockToken(err.to_string()))?
            .clone();

        let token = token.ok_or(CaptureError::NoTokenPresent)?;
        Todo::load(GitlabStorage::new(token)).map_err(|err| CaptureError::ErrorGettingTodoList(err))
    }

    #[export]
    fn _save_me(&mut self, _owner: TRef<TextureButton>) {
        godot_print!("Save me, SAVE me, SAAAAAAAAVE MEEEEEE");
    }
}

fn get_latest_todos(inbox: &String) -> String {
    let todos = inbox
        .split('\n')
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
    let todo_count = todos.len();

    todos
        .iter()
        .skip(todo_count - 4)
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_all_items_up_to_four() {
        let full_list = "- one
- two
- three
- four"
            .to_string();
        assert_eq!(full_list, get_latest_todos(&full_list));
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

        assert_eq!(expected, get_latest_todos(&full_list));
    }
}
