use gdnative::api::OS;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Button)]
pub struct Login;

#[methods]
impl Login {
    fn new(_owner: &Button) -> Self {
        Login
    }

    #[export]
    fn _button_pressed(&self, owner: TRef<Button>) {
        owner
            .get_tree()
            .map(|tree| unsafe { tree.assume_safe() })
            .map(|tree| {
                tree.change_scene("res://CaptureNote.tscn");
            });
        //        let url = "https://gitlab.example.com/oauth/authorize?client_id=APP_ID&redirect_uri=REDIRECT_URI&response_type=token&state=YOUR_UNIQUE_STATE_HASH&scope=REQUESTED_SCOPES";
        //        OS::godot_singleton().shell_open(url);
        //
        //
    }
}

#[cfg(test)]
mod tests {}
