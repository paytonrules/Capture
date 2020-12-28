use super::decoder::decode_content;
use anyhow::bail;
use ureq::json;

pub trait Storage {
    fn update(&self, inbox: &String) -> anyhow::Result<()>;
    fn load(&self) -> anyhow::Result<String>;
}

#[derive(Debug)]
pub struct GitlabStorage {
    pub token: String,
}

impl GitlabStorage {
    pub fn new(token: String) -> Self {
        GitlabStorage { token }
    }
}

impl Storage for GitlabStorage {
    fn update(&self, reminders: &String) -> anyhow::Result<()> {
        let content = json!({
            "branch": "master",
            "content": reminders,
            "commit_message": "Reminder(s) added from Capture app"
        });
        let response = ureq::put(
            "https://gitlab.com/api/v4/projects/3723174/repository/files/gtd%2Finbox%2Eorg",
        )
        .set("Authorization", &format!("Bearer {}", self.token))
        .send_json(content);

        match response.synthetic_error() {
            None => Ok(()),
            Some(error) => bail!("Error posting new content {}", error),
        }
    }

    fn load(&self) -> anyhow::Result<String> {
        let resp = ureq::get("https://gitlab.com/api/v4/projects/3723174/repository/files/gtd%2Finbox%2Eorg?ref=master")
            .set("Authorization", &format!("Bearer {}", self.token))
            .call();

        match resp.synthetic_error() {
            Some(error) => bail!("Response error {}", error),
            None => match decode_content(resp.into_json().unwrap()) {
                Ok(content) => Ok(content),
                Err(err) => Err(err.into()),
            },
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use thiserror::Error;

    #[derive(Debug, PartialEq, Error)]
    pub enum MockStorageError {
        #[error("{0}")]
        CantSave(String),
    }

    #[derive(Debug, Clone, Copy, Error)]
    pub enum MockError {
        #[error("Test Failed To Load")]
        TestFailedToLoad,
    }

    pub struct MockStorage {
        inbox: RefCell<String>,
        update_error: Option<String>,
        load_error: Option<MockError>,
    }

    impl MockStorage {
        pub fn new() -> Self {
            MockStorage {
                inbox: RefCell::new("".to_string()),
                update_error: None,
                load_error: None,
            }
        }

        pub fn inbox(&self) -> String {
            self.inbox.borrow().to_string()
        }

        pub fn with_update_error(mut self, error: &str) -> Self {
            self.update_error = Some(error.to_string());
            self
        }

        pub fn with_inbox(self, inbox: &str) -> Self {
            *self.inbox.borrow_mut() = inbox.to_string();
            self
        }

        pub fn with_load_error(mut self, error: MockError) -> Self {
            self.load_error = Some(error);
            self
        }

        pub fn as_rc(self) -> Rc<Self> {
            Rc::new(self)
        }
    }

    impl Storage for Rc<MockStorage> {
        fn update(&self, inbox: &String) -> anyhow::Result<()> {
            match &self.update_error {
                None => {
                    *self.inbox.borrow_mut() = inbox.to_string();
                    Ok(())
                }
                Some(update_error) => {
                    Err(MockStorageError::CantSave(update_error.to_string()).into())
                }
            }
        }

        fn load(&self) -> anyhow::Result<String> {
            match &self.load_error {
                None => Ok(self.inbox.borrow().to_string()),
                Some(err) => Err(err.clone().into()),
            }
        }
    }
}
