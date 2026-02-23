pub mod data;
pub mod error;
pub mod join;
pub mod render;
pub mod template;

pub use error::MailnirError;
pub type Result<T> = std::result::Result<T, MailnirError>;
