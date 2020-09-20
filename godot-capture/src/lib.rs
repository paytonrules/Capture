mod login;

use gdnative::prelude::*;
use login::Login;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
}

godot_init!(init);
