use crate::error::AppError;

mod i18n;
mod migrations;

pub(crate) fn handle_codegen(mode: &str) -> Result<(), AppError> {
    let mode = mode.trim().to_lowercase();

    match mode.as_str() {
        "" | "all" => {
            migrations::generate_migrations()?;
            i18n::generate_i18n()
        },
        "migrations" => migrations::generate_migrations(),
        "i18n" | "intl" => i18n::generate_i18n(),
        _ => Err(AppError::internal().with_message(format!("Invalid codegen mode: {mode}"))),
    }
}
