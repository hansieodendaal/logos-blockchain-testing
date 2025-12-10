use std::{env, path::PathBuf};

pub struct BinaryConfig {
    pub env_var: &'static str,
    pub binary_name: &'static str,
    pub fallback_path: &'static str,
    pub shared_bin_subpath: &'static str,
}

pub struct BinaryResolver;

impl BinaryResolver {
    pub fn resolve_path(config: &BinaryConfig) -> PathBuf {
        if let Some(path) = env::var_os(config.env_var) {
            return PathBuf::from(path);
        }
        if let Some(path) = Self::which_on_path(config.binary_name) {
            return path;
        }
        let shared_bin = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(config.shared_bin_subpath);
        if shared_bin.exists() {
            return shared_bin;
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../")
            .join(config.fallback_path)
    }

    fn which_on_path(bin: &str) -> Option<PathBuf> {
        let path_env = env::var_os("PATH")?;
        env::split_paths(&path_env)
            .map(|p| p.join(bin))
            .find(|candidate| candidate.is_file())
    }
}
