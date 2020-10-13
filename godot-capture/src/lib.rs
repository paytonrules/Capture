#![feature(proc_macro_hygiene, decl_macro)]

mod nodes;
mod oauth;
mod todo;

use gdnative::prelude::*;
use nodes::capture_note::Remember;
use nodes::login::Login;
use nodes::oauth::OAuthValidation;

#[macro_use]
extern crate rocket;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
    handle.add_class::<Remember>();
    handle.add_class::<OAuthValidation>();
}

godot_init!(init);
