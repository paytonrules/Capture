use gdnative::api::OS;
use gdnative::prelude::*;

const GITLAB_URI: &str = "https://gitlab.com/oauth/authorize?client_id=1ec97e4c1c7346edf5ddb514fdd6598e304957b40ca5368b1f191ffc906142ba&redirect_uri=http://127.0.0.1:8080/capture/&response_type=token&state=100&scope=api";

#[derive(NativeClass)]
#[inherit(Button)]
pub struct Login;

#[methods]
impl Login {
    fn new(_owner: &Button) -> Self {
        Login
    }

    #[export]
    fn _button_pressed(&self, _owner: TRef<Button>) {
        /*        owner
            .get_tree()
            .map(|tree| unsafe { tree.assume_safe() })
            .map(|tree| {
                tree.change_scene("res://CaptureNote.tscn");
        });*/

        OS::godot_singleton()
            .shell_open(GITLAB_URI)
            .expect("should open");
    }
}

#[cfg(test)]
mod tests {}
