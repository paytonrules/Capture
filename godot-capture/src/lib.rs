#![feature(proc_macro_hygiene, decl_macro)]

mod nodes;
mod oauth;

use gdnative::prelude::*;
use nodes::capture_note::Remember;
use nodes::listener::Listener;
use nodes::login::Login;

#[macro_use]
extern crate rocket;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
    handle.add_class::<Remember>();
    handle.add_class::<Listener>();
}

godot_init!(init);
