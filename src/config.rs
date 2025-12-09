use eyre::WrapErr as _;

const APP_NAME: &str = "xpost";

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Config {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl Config {
    pub fn load() -> eyre::Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("failed to read config from {}", path.display()))?;
        serde_json::from_str(&contents)
            .wrap_err_with(|| format!("failed to parse config from {}", path.display()))
    }

    pub fn save(&self) -> eyre::Result<()> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).wrap_err_with(|| {
                format!("failed to create config directory {}", parent.display())
            })?;
        }
        let contents = serde_json::to_string_pretty(self).wrap_err("failed to serialize config")?;
        std::fs::write(&path, contents)
            .wrap_err_with(|| format!("failed to write config to {}", path.display()))
    }

    fn path() -> eyre::Result<std::path::PathBuf> {
        let dirs = directories::ProjectDirs::from("", "", APP_NAME)
            .ok_or_else(|| eyre::eyre!("could not determine config directory"))?;
        Ok(dirs.config_dir().join("config.json"))
    }
}
