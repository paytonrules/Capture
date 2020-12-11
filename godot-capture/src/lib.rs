#![feature(proc_macro_hygiene, decl_macro)]

mod inbox;
mod nodes;
mod oauth;

use gdnative::prelude::*;
use nodes::capture_note::Remember;
use nodes::login::Login;
use oauth::save_token;
use std::ffi::CStr;
use std::os::raw::c_char;

#[macro_use]
extern crate rocket;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
    handle.add_class::<Remember>();
}

godot_init!(init);

#[no_mangle]
pub extern "C" fn logged_in(fragment: *const c_char) {
    let fragment = unsafe { CStr::from_ptr(fragment) }.to_str();
    let fragment = fragment.unwrap();
    let token = fragment
        .split('&')
        .find(|segment| segment.starts_with("access_token"));
    let token = token.unwrap().split('=').last().unwrap().to_string();

    save_token(token);
}

#[cfg(test)]
mod tests {
    use super::*;
    use oauth::{clear_token, get_token};
    use serial_test::serial;
    use std::ffi::CString;

    #[test]
    #[serial(accesses_token)]
    fn test_logged_in_sets_access_token() -> Result<(), Box<dyn std::error::Error>> {
        clear_token();

        let fragment =
            CString::new("access_token=passed_in_token&token_type=<ignore>&state=<ignore>");
        logged_in(fragment?.as_ptr());

        assert_eq!("passed_in_token", get_token()?);
        Ok(())
    }
}
