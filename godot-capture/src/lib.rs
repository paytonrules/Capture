use gdnative::prelude::*;
mod login;

fn init(handle: InitHandle) {
    handle.add_class::<login::Login>();
}

godot_init!(init);
