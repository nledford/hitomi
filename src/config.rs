//! Configuration for `chidori`

use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use clap::Args;
use default_struct_builder::DefaultBuilder;
use dialoguer::{Input, Select};
use dialoguer::theme::ColorfulTheme;
use serde::{Deserialize, Serialize};
use simplelog::{debug, error, info};

use crate::plex::Plex;

/// Default config file path where application config will be stored.
fn build_config_path() -> String {
    let config_dir = if let Ok(dir) = env::var("CONFIG_DIR") {
        PathBuf::from_str(&dir).expect("Error parsing `CONFIG_DIR`")
    } else {
        dirs::config_dir().expect("Could not fetch the user's configuration directory")
    }
        .join(env!("CARGO_PKG_NAME"));

    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    config_path.into_os_string().into_string().unwrap()
}

/// Represents the configuration file
#[derive(Args, Clone, Debug, DefaultBuilder, Deserialize, Serialize, PartialEq)]
pub struct Config {
    #[arg(long)]
    plex_token: String,
    #[arg(long)]
    plex_url: String,
    #[arg(long)]
    primary_section_id: i32,
    #[arg(long, default_value_t = String::from("."))]
    profiles_directory: String,
    #[serde(skip)]
    loaded: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plex_url: "http://127.0.0.1:32400".to_string(),
            plex_token: "PLEX_TOKEN".to_string(),
            primary_section_id: 0,
            profiles_directory: "./profiles".to_string(),
            loaded: false,
        }
    }
}

impl Config {
    pub fn get_plex_url(&self) -> &str {
        &self.plex_url
    }

    pub fn get_plex_token(&self) -> &str {
        &self.plex_token
    }

    pub fn get_primary_section_id(&self) -> i32 {
        self.primary_section_id
    }

    pub fn get_profiles_directory(&self) -> &str {
        &self.profiles_directory
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub async fn save_config(&self, config_path: Option<&str>) -> Result<()> {
        debug!("Saving config to disk...");

        let default_config_path = build_config_path();
        let config_path = if let Some(config_path) = config_path {
            Path::new(config_path)
        } else {
            Path::new(&default_config_path)
        };
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(config_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    pub async fn load_config(config_path: Option<&str>) -> Result<Self> {
        debug!("Loading config...");

        let default_config_path = build_config_path();
        let config_path = match config_path {
            None => Path::new(&default_config_path),
            Some(path) => Path::new(path),
        };

        if !config_path.exists() {
            return build_config_wizard().await;
        }

        let mut file = File::open(config_path)?;
        let mut config = String::default();
        file.read_to_string(&mut config)?;

        if let Ok(mut config) = serde_json::from_str::<Config>(&config) {
            config = config.loaded(true);
            Ok(config)
        } else {
            Ok(build_config_wizard().await?)
        }
    }
}

pub async fn delete_config_file() {
    tokio::fs::remove_file(build_config_path()).await.unwrap()
}

/// Wizard used by user to create an initial configuration file
pub async fn build_config_wizard() -> Result<Config> {
    info!("Config file not found. Checking for environment variables...");

    let profiles_directory = if let Ok(dir) = env::var("PROFILES_DIRECTORY") {
        dir
    } else {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter a directory to store your profiles:")
            .default("./profiles".to_string())
            .interact_text()?
            .to_string()
    };

    let (plex_url, plex_token) = loop {
        let plex_url = if let Ok(plex_url) = env::var("PLEX_URL") {
            plex_url
        } else {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your plex URL:")
                .interact_text()?
                .to_string()
        };

        let plex_token = if let Ok(plex_token) = env::var("PLEX_TOKEN") {
            plex_token
        } else {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your plex token:")
                .interact_text()?
                .to_string()
        };

        info!("Testing connection to plex. Please wait...");
        if Plex::new_for_config(&plex_url, &plex_token).await.is_ok() {
            info!("Success!");
            break (plex_url, plex_token);
        } else {
            error!("Could not connect to plex. Please re-enter your URL and token.")
        }
    };

    let primary_section_id = if let Ok(id) = env::var("PRIMARY_SECTION_ID") {
        id.parse::<i32>()
    } else {
        let plex = Plex::new_for_config(&plex_url, &plex_token).await?;
        let sections = plex.get_music_sections();
        let titles = sections
            .iter()
            .map(|x| x.title.to_owned())
            .collect::<Vec<String>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your music library:")
            .default(0)
            .items(&titles)
            .interact()?;
        sections[selection]
            .id()
            .parse::<i32>()
    }.expect("Could not parse section id");

    let config = Config::default()
        .profiles_directory(profiles_directory)
        .plex_url(plex_url)
        .plex_token(plex_token)
        .primary_section_id(primary_section_id)
        .loaded(true);
    let data = serde_json::to_string_pretty(&config)?;

    let mut file = File::create(build_config_path())?;
    file.write_all(data.as_bytes())?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    static PROFILES_DIRECTORY: &str = "/data";

    async fn build_test_config_path() -> PathBuf {
        let path = Path::new("./data/test");
        tokio::fs::create_dir_all(path).await.unwrap();
        path.join("test-config.json")
    }

    #[tokio::test]
    async fn test_saving_config() {
        let config = Config::default().profiles_directory(PROFILES_DIRECTORY.to_owned());
        config
            .save_config(Some(build_test_config_path().await.to_str().unwrap()))
            .await
            .unwrap();

        let config = Config::load_config(Some(build_test_config_path().await.to_str().unwrap()))
            .await
            .unwrap();
        assert_eq!(config.profiles_directory, PROFILES_DIRECTORY);
    }
}
