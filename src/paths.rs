use std::path::PathBuf;

/// Default path to config
/// Windows: %APPDATA%\tiramisu\config.toml
/// Unix: $XDG_CONFIG_HOME/tiramisu/config.toml or ~/.config/tiramisu/config.toml
pub fn config() -> PathBuf {
    dirs::config_dir()
        .unwrap_or(PathBuf::from("."))
        .join("tiramisu")
        .join("config.toml")
}

/// Default path to logs
/// Windows: %LOCALAPPDATA%\tiramisu\tiramisu.log
/// Unix: $XDG_CACHE_HOME/tiramisu/tiramisu.log or ~/.cache/tiramisu/tiramisu.log
pub fn logs() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or(PathBuf::from("."))
        .join("tiramisu")
        .join("config.toml")
}
