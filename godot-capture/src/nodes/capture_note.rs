use crate::todo::{GitlabStorage, Todo};
use gdnative::api::TextureButton;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(TextureButton)]
pub struct Remember {
    todo: Todo<GitlabStorage>,
}

#[methods]
impl Remember {
    fn new(_owner: &TextureButton) -> Self {
        Remember {
            todo: Todo::new(GitlabStorage::new("where is the token".to_string())),
        }
    }

    #[export]
    fn _ready(&mut self, _owner: TRef<TextureButton>) {
        godot_print!("Ready Santa");
        self.todo = Todo::load(GitlabStorage::new("where is the token".to_string()))
            .expect("Some todo stuff");

        godot_print!("{}", self.todo.inbox);
    }

    #[export]
    fn _save_me(&mut self, _owner: TRef<TextureButton>) {
        godot_print!("Save me, SAVE me, SAAAAAAAAVE MEEEEEE");
    }
}
