use nx_error::prelude::*;

#[error]
pub enum {{ shortname | pascal_case }}Error {
    #[error(message = "Internal failure")]
    Internal,
}
