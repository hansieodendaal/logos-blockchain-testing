#![allow(dead_code)]

use std::path::Path;

use nomos_tracing::logging::local::FileConfig;

/// Configure tracing logger to write into `NOMOS_LOG_DIR` if set, else into the
/// provided base dir.
pub fn configure_logging<F>(base_dir: &Path, prefix: &str, set_logger: F)
where
    F: FnOnce(FileConfig),
{
    if let Ok(env_dir) = std::env::var("NOMOS_LOG_DIR") {
        let log_dir = std::path::PathBuf::from(env_dir);
        let _ = std::fs::create_dir_all(&log_dir);
        set_logger(FileConfig {
            directory: log_dir,
            prefix: Some(prefix.into()),
        });
    } else {
        set_logger(FileConfig {
            directory: base_dir.to_owned(),
            prefix: Some(prefix.into()),
        });
    }
}
