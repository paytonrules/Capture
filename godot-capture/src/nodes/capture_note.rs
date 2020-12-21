use crate::inbox::{GitlabStorage, Inbox, InboxError, Storage};
use crate::oauth::AuthState;
use crate::oauth::TokenRetriever;
use gdnative::api::{AcceptDialog, TextEdit, TextureButton};
use gdnative::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Token Not Available")]
    TokenFailure,

    #[error("Error getting inbox: {0}")]
    ErrorGettingInbox(#[from] InboxError),
}

#[derive(NativeClass)]
#[inherit(TextureButton)]
pub struct Remember {
    inbox: Option<Inbox<GitlabStorage>>,
}

#[methods]
impl Remember {
    fn new(_owner: &TextureButton) -> Self {
        Remember { inbox: None }
    }

    #[export]
    fn _ready(&mut self, owner: TRef<TextureButton>) {
        self.inbox = create_storage(&AuthState::get())
            .and_then(|storage| load_inbox(storage))
            .or_else(|err| {
                display_error(owner, &err);
                Err(err)
            })
            .ok();

        match &self.inbox {
            Some(inbox) => update_view(owner, &inbox.reminders),
            None => clear_list(owner),
        }
    }

    #[export]
    fn _save_me(&mut self, owner: TRef<TextureButton>) {
        let new_reminder = new_reminder_window(owner);

        if let Some(inbox) = &mut self.inbox {
            match save_new_reminder(inbox, &new_reminder.text().to_string()) {
                Ok(_) => update_view(owner, &inbox.reminders),
                Err(err) => display_error(owner, &err),
            }
        }
    }

    #[export]
    fn _button_down(&self, owner: TRef<TextureButton>) {
        owner.set_position(pressed_button(owner.position()), false);
    }

    #[export]
    fn _button_up(&self, owner: TRef<TextureButton>) {
        owner.set_position(released_button(owner.position()), false);
    }
}

fn display_error(owner: TRef<TextureButton>, err: &CaptureError) {
    let dialog = AcceptDialog::new();
    dialog.set_text(err.to_string());
    let dialog = unsafe { dialog.assume_shared() };
    owner.add_child(dialog, false);
    let dialog = unsafe { dialog.assume_safe() };
    dialog.popup_centered(Vector2::new(0.0, 0.0));
}

fn update_view(owner: TRef<TextureButton>, inbox: &str) {
    let inbox_view = owner
        .get_node("/root/CaptureNote/CenterContainer/VBoxContainer/Recent Todos")
        .map(|node| unsafe { node.assume_safe() })
        .and_then(|node| node.cast::<Label>())
        .expect("Recent Reminders node is missing");
    inbox_view.set_text(truncate_to_latest_reminders(inbox));
    let new_reminder_window = new_reminder_window(owner);
    new_reminder_window.set_text("");
}

fn clear_list(owner: TRef<TextureButton>) {
    let inbox_view = owner
        .get_node("/root/CaptureNote/CenterContainer/VBoxContainer/Recent Todos")
        .map(|node| unsafe { node.assume_safe() })
        .and_then(|node| node.cast::<Label>())
        .expect("Recent Reminders node is missing");
    inbox_view.set_text("");
}

fn new_reminder_window(owner: TRef<TextureButton>) -> TRef<TextEdit> {
    owner
        .get_node("/root/CaptureNote/CenterContainer/VBoxContainer/New Todo")
        .map(|node| unsafe { node.assume_safe() })
        .and_then(|node| node.cast::<TextEdit>())
        .expect("New Reminder node is missing")
}

const BUTTON_PRESS_MOVEMENT: f32 = 2.0;
fn pressed_button(mut position: Vector2) -> Vector2 {
    position.y = position.y + BUTTON_PRESS_MOVEMENT;
    position
}

fn released_button(mut position: Vector2) -> Vector2 {
    position.y = position.y - BUTTON_PRESS_MOVEMENT;
    position
}

fn truncate_to_latest_reminders(all_reminders: &str) -> String {
    let reminder_list = all_reminders
        .split('\n')
        .map(|str| str.to_string())
        .collect::<Vec<String>>();
    let reminder_count = reminder_list.len();

    if reminder_count > 4 {
        reminder_list
            .iter()
            .skip(reminder_count - 4)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join("\n")
            .to_string()
    } else {
        all_reminders.to_string()
    }
}

fn create_storage<T>(token_retriever: &T) -> Result<GitlabStorage, CaptureError>
where
    T: TokenRetriever,
{
    let token = token_retriever.token().ok_or(CaptureError::TokenFailure)?;

    Ok(GitlabStorage::new(token.to_string()))
}

fn load_inbox<T: Storage>(storage: T) -> Result<Inbox<T>, CaptureError> {
    Inbox::load(storage).map_err(|err| CaptureError::ErrorGettingInbox(err))
}

fn save_new_reminder<T: Storage>(inbox: &mut Inbox<T>, reminder: &str) -> Result<(), CaptureError> {
    inbox.save(&reminder.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbox::{MockError, MockStorage};
    use crate::oauth::TokenRetriever;
    use std::rc::Rc;

    #[test]
    fn take_all_items_up_to_four() {
        let full_list = "- one
- two
- three
- four"
            .to_string();
        assert_eq!(full_list, truncate_to_latest_reminders(&full_list));
    }

    #[test]
    fn take_only_the_last_four() {
        let full_list = "- skip
- one
- two
- three
- four"
            .to_string();
        let expected = "- one
- two
- three
- four"
            .to_string();

        assert_eq!(expected, truncate_to_latest_reminders(&full_list));
    }

    #[test]
    fn take_all_the_entries_if_there_are_lt_four() {
        let full_list = "- one
- two
- three"
            .to_string();
        let expected = "- one
- two
- three"
            .to_string();

        assert_eq!(expected, truncate_to_latest_reminders(&full_list));
    }

    struct StubTokenRetriever {
        token: Option<String>,
    }

    impl StubTokenRetriever {
        fn new_with_token(token: &str) -> Self {
            StubTokenRetriever {
                token: Some(token.to_owned()),
            }
        }

        fn new_without_token() -> Self {
            StubTokenRetriever { token: None }
        }
    }

    impl TokenRetriever for StubTokenRetriever {
        fn token(&self) -> Option<String> {
            self.token.clone()
        }
    }

    #[test]
    fn when_a_token_is_present_create_storage() -> Result<(), Box<dyn std::error::Error>> {
        let token_retriever = StubTokenRetriever::new_with_token("token");

        let storage = create_storage(&token_retriever)?;

        assert_eq!("token", storage.token);
        Ok(())
    }

    #[test]
    fn when_a_token_is_not_present() -> Result<(), Box<dyn std::error::Error>> {
        let token_retriever = StubTokenRetriever::new_without_token();
        let storage = create_storage(&token_retriever);

        match storage {
            Err(CaptureError::TokenFailure) => assert!(true, "correct error"),
            Ok(token) => assert!(false, println!("unexpected Ok result - {:?}", token)),
            Err(err) => assert!(false, println!("unexpected error thrown {:?}", err)),
        }

        Ok(())
    }

    #[test]
    fn load_todos_from_storage() {
        let storage = Rc::new(MockStorage::new().with_inbox("-first\nsecond"));

        let todos = load_inbox(storage);
        assert!(todos.is_ok());
        assert_eq!("-first\nsecond", todos.unwrap().reminders);
    }

    #[test]
    fn map_load_todos_failure_to_capture_error() {
        let storage = Rc::new(MockStorage::new().with_load_error(MockError::TestFailedToLoad));

        let todos = load_inbox(storage);
        match todos {
            Err(CaptureError::ErrorGettingInbox(err)) => match err {
                InboxError::FailedToLoad(sub_err) => match sub_err.downcast::<MockError>() {
                    Ok(MockError::TestFailedToLoad) => assert!(true, "correct error"),
                    _ => assert!(false, "incorrect error"),
                },
                _ => assert!(false, "incorrect error"),
            },
            Ok(_) => assert!(false, println!("unexpected Ok result")),
            Err(err) => assert!(false, println!("unexpected error thrown {:?}", err)),
        }
    }

    #[test]
    fn save_new_todo_saves() -> Result<(), Box<dyn std::error::Error>> {
        let storage = Rc::new(MockStorage::new().with_inbox("- one"));
        let mut todos = Inbox::load(storage)?;

        save_new_reminder(&mut todos, "two")?;

        assert_eq!("- one\n- two", todos.reminders);
        Ok(())
    }

    #[test]
    fn pressed_button_returns_a_lower_position() {
        let position = Vector2::new(10.0, 14.0);

        let new_position = pressed_button(position);

        assert_eq!(10.0, new_position.x);
        assert_eq!(14.0 + BUTTON_PRESS_MOVEMENT, new_position.y);
    }

    #[test]
    fn released_button_returns_a_higher_position() {
        let position = Vector2::new(10.0, 16.0);

        let new_position = released_button(position);

        assert_eq!(10.0, new_position.x);
        assert_eq!(16.0 - BUTTON_PRESS_MOVEMENT, new_position.y);
    }
}
