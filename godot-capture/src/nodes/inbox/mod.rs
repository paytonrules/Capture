mod decoder;
mod inbox;
pub mod storage;
pub use inbox::Inbox;
pub use inbox::InboxError;
pub use storage::{GitlabStorage, Storage};
