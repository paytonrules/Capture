mod capture_note;
mod login;
mod oauth;

use capture_note::Remember;
use gdnative::prelude::*;
use login::Login;
use oauth::Listener;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
    handle.add_class::<Remember>();
    handle.add_class::<Listener>();
}

godot_init!(init);
