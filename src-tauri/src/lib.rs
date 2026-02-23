pub mod data;
pub mod error;
pub mod join;
pub mod render;
pub mod template;
pub mod validate;

pub use error::MailnirError;
pub use validate::{EntryResult, JoinFailureDetail, ValidationIssue, ValidationReport};
pub type Result<T> = std::result::Result<T, MailnirError>;
