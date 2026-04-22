use std::path::PathBuf;

pub(crate) fn app_data_dir() -> PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexus")
        .join(env!("CARGO_PKG_NAME"));

    if !dir.exists() {
        std::fs::create_dir_all(&dir).ok();
    }

    dir
}
