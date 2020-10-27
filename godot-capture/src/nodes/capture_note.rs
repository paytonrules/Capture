use crate::todo::{GitlabStorage, Todo};
use gdnative::api::TextureButton;
use gdnative::prelude::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref TOKEN: Mutex<Option<String>> = Mutex::new(None);
}

pub fn save_token(token: String) {
    *TOKEN.lock().unwrap() = Some(token);
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
    fn _ready(&mut self, _owner: TRef<TextureButton>) {
        self.todo = self.load_todos().ok();
    }

    fn load_todos(&self) -> Result<Todo<GitlabStorage>, crate::todo::TodoError> {
        let token = TOKEN
            .try_lock()
            .map_err(|err| crate::todo::TodoError::CouldNotSaveTodo("doom".to_string()))?
            .clone();

        let token = token.ok_or(crate::todo::TodoError::CouldNotSaveTodo("nope".to_string()))?;
        Todo::load(GitlabStorage::new(token))
    }

    #[export]
    fn _save_me(&mut self, _owner: TRef<TextureButton>) {
        godot_print!("Save me, SAVE me, SAAAAAAAAVE MEEEEEE");
    }
}
