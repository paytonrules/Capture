mod capture_note;
mod login;

use capture_note::Remember;
use gdnative::prelude::*;
use login::Login;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
    handle.add_class::<Remember>();
}

godot_init!(init);
