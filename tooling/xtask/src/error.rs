/// High-level errors for the `xtask` automation suite.
#[nx_error::error]
pub(crate) enum AppError {
    #[error(message = "A filesystem operation failed", code = "IO_ERROR", source = std::io::Error)]
    Io,

    #[error(message = "A cargo generate operation failed", code = "CARGO_GENERATE_ERROR")]
    CargoGenerate,

    #[error(message = "Code generation failed", code = "CODEGEN_ERROR")]
    Codegen,

    #[error(message = "Text formatting failure", code = "FORMAT_ERROR", source = std::fmt::Error)]
    Format,

    #[error(message = "Parsing failure", code = "PARSE_ERROR")]
    Parse,

    #[error(message = "Internal error", code = "INTERNAL_ERROR")]
    Internal,
}
