use crate::nodes::capture_note::save_token;
use gdnative::api::OS;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Button)]
pub struct Login {
    login_url: Option<String>,
}

#[methods]
impl Login {
    fn new(_owner: &Button) -> Self {
        Login { login_url: None }
    }

    #[export]
    fn _login_url_received(&mut self, _owner: TRef<Button>, url: Variant) {
        self.login_url = url.try_to_string();
    }

    #[export]
    fn _button_pressed(&self, _owner: TRef<Button>) {
        if let Some(login_url) = &self.login_url {
            OS::godot_singleton()
                .shell_open(login_url)
                .expect("should open");
        }
    }

    #[export]
    fn _token_received(&self, owner: TRef<Button>, token: String) {
        owner
            .get_tree()
            .map(|tree| unsafe { tree.assume_safe() })
            .map(|tree| {
                save_token(token);
                tree.change_scene("res://CaptureNote.tscn")
                    .expect("Should change scene");
            });
    }
}

#[cfg(test)]
mod tests {}
