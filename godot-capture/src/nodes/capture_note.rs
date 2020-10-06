use gdnative::api::TextureButton;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(TextureButton)]
pub struct Remember;

#[methods]
impl Remember {
    fn new(_owner: &TextureButton) -> Self {
        Remember
    }

    #[export]
    fn _save_me(&self, _owner: TRef<TextureButton>) {
        godot_print!("saved the remember");
    }
}
