#![feature(proc_macro_hygiene, decl_macro)]

mod capture_note;
mod login;
mod oauth;

use capture_note::Remember;
use gdnative::prelude::*;
use login::Login;

#[macro_use]
extern crate rocket;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
    handle.add_class::<Remember>();
    handle.add_class::<oauth::Listener>();
}

godot_init!(init);
