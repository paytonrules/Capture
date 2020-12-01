#![feature(proc_macro_hygiene, decl_macro)]

mod inbox;
mod nodes;
mod oauth;

use gdnative::prelude::*;
use nodes::capture_note::Remember;
use nodes::login::Login;
use nodes::oauth::OAuthValidation;
use std::ffi::CStr;
use std::os::raw::c_char;

#[macro_use]
extern crate rocket;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
    handle.add_class::<Remember>();
    handle.add_class::<OAuthValidation>();
}

godot_init!(init);

#[no_mangle]
pub extern "C" fn logged_in(fragment: *const c_char) {
    let fragment = unsafe { CStr::from_ptr(fragment) };
    println!("Logged In fragment: {:#?}", fragment.to_str());
}
