pub mod error;

use crate::error::{{ shortname | pascal_case }}Error;

pub async fn run() -> Result<(), {{ shortname | pascal_case }}Error> {
    Ok(())
}