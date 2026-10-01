use std::path::PathBuf;

const APP_DIR_NAME: &str = "local-recorder";

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR_NAME)
}

pub fn data_dir() -> PathBuf {
    config_dir().join("data")
}

pub fn db_path() -> PathBuf {
    data_dir().join("app.db")
}

pub fn models_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("LRA_MODELS_DIR") {
        if !custom.trim().is_empty() {
            return PathBuf::from(custom);
        }
    }

    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    project_root.join("models")
}

pub fn outputs_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Recordings_Summary")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_have_expected_suffixes() {
        assert!(config_dir().ends_with("local-recorder"));
        assert!(data_dir().ends_with("local-recorder/data"));
        assert!(db_path().ends_with("local-recorder/data/app.db"));
        assert!(models_dir().ends_with("models"));
        assert!(outputs_dir().ends_with("Recordings_Summary"));
    }
}
