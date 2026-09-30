use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProjectConfig {
    pub subdirs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub projects_dir: PathBuf,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
    #[serde(default = "default_ui_port")]
    pub ui_port: u16,
}

fn default_ui_port() -> u16 {
    7000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            projects_dir: PathBuf::new(),
            ignore: Vec::new(),
            projects: HashMap::new(),
            ui_port: default_ui_port(),
        }
    }
}

impl Config {
    pub fn resolve_ui_port(&self, port: Option<u16>) -> Result<u16> {
        let port = port.unwrap_or(self.ui_port);
        anyhow::ensure!(port != 0, "UI port must be between 1 and 65535");
        Ok(port)
    }

    pub fn is_configured(&self) -> bool {
        !self.projects_dir.as_os_str().is_empty()
    }

    pub fn path() -> PathBuf {
        dirs::config_dir().unwrap().join("bolt").join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Allowed subdirs for a project, or None if unrestricted
    pub fn subdirs_for(&self, project: &str) -> Option<&Vec<String>> {
        self.projects
            .get(project)
            .map(|p| &p.subdirs)
            .filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_configs_keep_default_port() {
        let config: Config = toml::from_str("projects_dir = '/tmp/projects'").unwrap();
        assert_eq!(config.resolve_ui_port(None).unwrap(), 7000);
        assert_eq!(Config::default().ui_port, 7000);
    }

    #[test]
    fn saved_port_is_used_unless_overridden() {
        let config = Config {
            ui_port: 8080,
            ..Config::default()
        };
        let saved = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&saved).unwrap();
        assert_eq!(loaded.resolve_ui_port(None).unwrap(), 8080);
        assert_eq!(loaded.resolve_ui_port(Some(9000)).unwrap(), 9000);
        assert_eq!(loaded.ui_port, 8080);
    }

    #[test]
    fn zero_port_is_rejected() {
        let config = Config {
            ui_port: 0,
            ..Config::default()
        };
        assert!(config.resolve_ui_port(None).is_err());
        assert_eq!(config.resolve_ui_port(Some(8080)).unwrap(), 8080);
    }
}
