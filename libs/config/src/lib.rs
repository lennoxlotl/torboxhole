use eyre::eyre;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::io::Write;
use std::path::Path;
use std::process::exit;

/// Loads a configuration file at a given path, saves and loads default if config is not present.
///
/// ### Arguments
/// * `path` - Path to load config from
/// * `default` - Default config content
/// * `exit_default` - Exit program on default creation to allow pre-configuration
pub fn load_config_at<T, P>(path: P, default: String, exit_default: bool) -> eyre::Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path> + Debug,
{
    if !std::fs::exists(&path)? {
        std::fs::File::create(&path)?
            .write_all(default.into_bytes().as_slice())
            .map_err(|e| eyre!("failed to write default config: {e}"))?;

        // Exit with code 0 if requested
        if exit_default {
            log::error!(
                "Default config at path {path:?} was created, please change to your liking before continuing",
            );
            exit(0)
        }
    }

    let file = std::fs::read_to_string(&path)?;
    toml::from_str(file.as_str()).map_err(|e| eyre!("failed to parse config: {e}"))
}
