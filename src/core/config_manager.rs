use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get the user data directory for AgentX
/// - macOS: ~/.agentx/
/// - Windows: %APPDATA%\agentx\
/// - Linux: ~/.config/agentx/
pub fn get_user_data_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        Ok(home.join(".agentx"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get AppData directory"))?;
        Ok(appdata.join("agentx"))
    }

    #[cfg(target_os = "linux")]
    {
        let config = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?;
        Ok(config.join("agentx"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}

/// Get the config file path in the user data directory
pub fn get_user_config_path() -> Result<PathBuf> {
    Ok(get_user_data_dir()?.join("config.json"))
}

/// Initialize user config directory and config file
/// If config file doesn't exist, create it from the embedded default config
pub fn initialize_user_config() -> Result<PathBuf> {
    let user_data_dir = get_user_data_dir()?;
    let config_path = get_user_config_path()?;

    // Create user data directory if it doesn't exist
    if !user_data_dir.exists() {
        log::info!("Creating user data directory: {:?}", user_data_dir);
        std::fs::create_dir_all(&user_data_dir)
            .with_context(|| format!("Failed to create directory: {:?}", user_data_dir))?;
    }

    // If config file doesn't exist, create it from embedded default
    if !config_path.exists() {
        log::info!("Config file not found, creating from embedded default: {:?}", config_path);

        let default_config = crate::assets::get_default_config()
            .ok_or_else(|| anyhow::anyhow!("Failed to get embedded default config"))?;

        std::fs::write(&config_path, default_config)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        log::info!("Created default config file at: {:?}", config_path);
    } else {
        log::info!("Using existing config file: {:?}", config_path);
    }

    Ok(config_path)
}

/// Load config from user data directory
/// Falls back to embedded default if file doesn't exist or is invalid
pub fn load_user_config() -> Result<crate::core::config::Config> {
    let config_path = initialize_user_config()?;

    let config_content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

    let config: crate::core::config::Config = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

    Ok(config)
}
