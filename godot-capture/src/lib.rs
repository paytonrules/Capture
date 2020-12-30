#![feature(proc_macro_hygiene, decl_macro)]
mod nodes;

use gdnative::prelude::*;
use itertools::Itertools;
use nodes::capture_note::Remember;
use nodes::login::Login;
use nodes::oauth::{AuthState, TokenError, TokenReceiver};
use std::collections::HashMap;
use std::ffi::CStr;
use std::num::ParseIntError;
use std::os::raw::c_char;
use std::str::Utf8Error;
use thiserror::Error;

fn init(handle: InitHandle) {
    handle.add_class::<Login>();
    handle.add_class::<Remember>();
}

godot_init!(init);

#[no_mangle]
pub extern "C" fn logged_in(fragment: *const c_char) {
    if let Err(err) = logged_in_with_auth_state(AuthState::get(), fragment) {
        println!("Error saving access token {:?}, {}", fragment, err);
    }
}

#[derive(Error, Debug, PartialEq)]
enum LoginError {
    #[error("Fragment is null")]
    NullFragment,

    #[error("Cannot convert the passed in fragment to a string, should be impossible {0}")]
    CannotConvertFragmentToString(Utf8Error),

    #[error("Cannot save token {0}")]
    CannotSaveToken(TokenError),

    #[error("Access Token is missing or invalid {0}")]
    InvalidAccessToken(String),

    #[error("State is missing or invalid {0}")]
    InvalidState(String),

    #[error("State param is present but could not be parsed to a number")]
    StateIsNotAValidNumber(String, ParseIntError, String),
}

fn logged_in_with_auth_state<T>(receiver: T, fragment: *const c_char) -> Result<(), LoginError>
where
    T: TokenReceiver,
{
    if fragment.is_null() {
        Err(LoginError::NullFragment)
    } else {
        let fragment = unsafe { CStr::from_ptr(fragment) }
            .to_str()
            .map_err(|err| LoginError::CannotConvertFragmentToString(err))?;

        let segments = fragment
            .split("&")
            .map(|pair| pair.split("=").collect_tuple())
            .filter(|tuple| tuple.is_some())
            .map(|tuple| tuple.unwrap())
            .collect::<HashMap<&str, &str>>();

        let token = segments
            .get("access_token")
            .ok_or(LoginError::InvalidAccessToken(fragment.into()))?;

        let state = segments
            .get("state")
            .ok_or(LoginError::InvalidState(fragment.into()))?;

        let state = state.parse::<i16>().map_err(|err| {
            LoginError::StateIsNotAValidNumber(state.to_string(), err, fragment.into())
        })?;

        receiver
            .token_received(token, state)
            .map_err(|err| LoginError::CannotSaveToken(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::oauth::token::tests::MockTokenReceiver;
    use crate::nodes::oauth::TokenError;
    use std::ffi::CString;
    use std::rc::Rc;

    fn fragment_with_token_and_state(token: &str, state: i16) -> String {
        format!("access_token={}&token_type=<ignore>&state={}", token, state)
    }

    #[test]
    fn logged_in_sets_access_token_and_state() -> Result<(), Box<dyn std::error::Error>> {
        let state = 104;
        let token = "passed_in_token";
        let url = fragment_with_token_and_state(token, state);

        let fragment = CString::new(url);
        let token_receiver = Rc::new(MockTokenReceiver::new_with_state(state));

        let result = logged_in_with_auth_state(Rc::clone(&token_receiver), fragment?.as_ptr());

        assert_eq!(Ok(()), result);
        assert_eq!(Some(state), token_receiver.received_state());
        assert_eq!(
            Some("passed_in_token".to_string()),
            token_receiver.received_token()
        );
        Ok(())
    }

    #[test]
    fn error_when_fragment_is_null() {
        let token_receiver = Rc::new(MockTokenReceiver::new_with_state(1));

        let result = logged_in_with_auth_state(Rc::clone(&token_receiver), std::ptr::null());

        assert_eq!(
            Err(LoginError::NullFragment) as Result<(), LoginError>,
            result
        );
        assert_eq!(None, token_receiver.received_state());
    }

    #[test]
    fn token_is_unchanged_when_fragment_has_no_seperator() -> Result<(), Box<dyn std::error::Error>>
    {
        let token_receiver = Rc::new(MockTokenReceiver::new_with_state(1));

        let result = logged_in_with_auth_state(
            Rc::clone(&token_receiver),
            CString::new("bad fragment")?.as_ptr(),
        );

        assert_eq!(
            Err(LoginError::InvalidAccessToken("bad fragment".to_string()))
                as Result<(), LoginError>,
            result
        );
        assert_eq!(None, token_receiver.received_token());
        Ok(())
    }

    #[test]
    fn token_is_unchanged_when_token_is_malformed() -> Result<(), Box<dyn std::error::Error>> {
        let token_receiver = Rc::new(MockTokenReceiver::new_with_state(100));
        let invalid_fragment = "access_token&state=100";

        let result = logged_in_with_auth_state(
            Rc::clone(&token_receiver),
            CString::new(invalid_fragment)?.as_ptr(),
        );

        assert_eq!(
            Err(LoginError::InvalidAccessToken(invalid_fragment.to_string()))
                as Result<(), LoginError>,
            result
        );
        assert_eq!(None, token_receiver.received_token());
        Ok(())
    }

    #[test]
    fn error_on_received_token_is_propigated() -> Result<(), Box<dyn std::error::Error>> {
        let token_receiver = Rc::new(MockTokenReceiver::new_with_state(1));
        let fragment_with_mismatched_state = fragment_with_token_and_state("token", 100);

        let result = logged_in_with_auth_state(
            Rc::clone(&token_receiver),
            CString::new(fragment_with_mismatched_state)?.as_ptr(),
        );

        assert_eq!(
            Err(LoginError::CannotSaveToken(TokenError::StateDoesntMatch))
                as Result<(), LoginError>,
            result
        );
        Ok(())
    }

    #[test]
    fn error_when_the_query_string_has_extra_equal_signs() -> Result<(), Box<dyn std::error::Error>>
    {
        let token_receiver = Rc::new(MockTokenReceiver::new_with_state(1));
        let invalid_access_token = "token=token=jimmy&state=1";

        let result = logged_in_with_auth_state(
            Rc::clone(&token_receiver),
            CString::new(invalid_access_token)?.as_ptr(),
        );

        assert_eq!(
            Err(LoginError::InvalidAccessToken(invalid_access_token.into())),
            result
        );

        Ok(())
    }

    #[test]
    fn error_when_state_is_invalid() -> Result<(), Box<dyn std::error::Error>> {
        let token_receiver = Rc::new(MockTokenReceiver::new_with_state(1));
        let invalid_state = "access_token=token&state";

        let result = logged_in_with_auth_state(
            Rc::clone(&token_receiver),
            CString::new(invalid_state)?.as_ptr(),
        );

        assert_eq!(Err(LoginError::InvalidState(invalid_state.into())), result);

        Ok(())
    }

    #[test]
    fn error_when_state_is_not_a_number() -> Result<(), Box<dyn std::error::Error>> {
        let token_receiver = Rc::new(MockTokenReceiver::new_with_state(1));
        let invalid_state = "access_token=token&state=not-a-number";
        let expected_error = "not-a-number".parse::<i16>();

        let result = logged_in_with_auth_state(
            Rc::clone(&token_receiver),
            CString::new(invalid_state)?.as_ptr(),
        );

        assert_eq!(
            Err(LoginError::StateIsNotAValidNumber(
                "not-a-number".into(),
                expected_error.unwrap_err(),
                invalid_state.into(),
            )),
            result
        );

        Ok(())
    }
}
