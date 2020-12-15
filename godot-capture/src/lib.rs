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
    if !fragment.is_null() {
        if let Ok(fragment) = unsafe { CStr::from_ptr(fragment) }.to_str() {
            save_access_token(fragment);
        }
    }
}

fn save_access_token(fragment: &str) -> anyhow::Result<()> {
    match fragment
        .split('&')
        .find(|segment| segment.starts_with("access_token"))
        .and_then(|pair| pair.split('=').nth(1))
    {
        Some(token) => {
            save_token(token.to_string());
            Ok(())
        }
        None => anyhow::bail!("Error parsing access token"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oauth::{clear_token, get_token, TokenError};
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

    #[test]
    #[serial(access_token)]
    fn test_null_ptr_handling() {
        clear_token();

        logged_in(std::ptr::null());

        assert_eq!(TokenError::NoTokenPresent, get_token().unwrap_err())
    }

    #[test]
    #[serial(access_token)]
    fn test_token_is_unchanged_when_access_token_is_not_present(
    ) -> Result<(), Box<dyn std::error::Error>> {
        clear_token();

        logged_in(CString::new("no_token")?.as_ptr());

        assert_eq!(TokenError::NoTokenPresent, get_token().unwrap_err());
        Ok(())
    }

    #[test]
    #[serial(access_token)]
    fn test_token_is_unchanged_when_token_is_malformed() -> Result<(), Box<dyn std::error::Error>> {
        clear_token();

        logged_in(CString::new("access_token&state=100")?.as_ptr());

        assert_eq!(TokenError::NoTokenPresent, get_token().unwrap_err());
        Ok(())
    }
}
