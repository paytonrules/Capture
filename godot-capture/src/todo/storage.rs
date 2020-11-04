use super::decoder::decode_content;
use anyhow::bail;
use ureq::json;

pub trait Storage {
    fn update(&self, inbox: &String) -> anyhow::Result<()>;
    fn load(&self) -> anyhow::Result<String>;
}

#[derive(Debug)]
pub struct GitlabStorage {
    token: String,
}

impl GitlabStorage {
    pub fn new(token: String) -> Self {
        GitlabStorage { token }
    }
}

impl Storage for GitlabStorage {
    fn update(&self, inbox: &String) -> anyhow::Result<()> {
        let content = json!({
            "branch": "master",
            "content": inbox,
            "commit_message": "Todo added from Capture app"
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
