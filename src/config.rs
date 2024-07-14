//! Configuration for `hitomi`

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};

use crate::plex::types::{PlexToken, PlexUrl};
use crate::plex::PlexClient;
use anyhow::Result;
use clap::Args;
use derive_builder::Builder;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use serde::{Deserialize, Serialize};
use simplelog::{debug, error, info};

/// Default config file path where application config will be stored.
fn build_config_path() -> String {
    let config_dir = if let Ok(dir) = env::var("CONFIG_DIR") {
        PathBuf::from_str(&dir).expect("Error parsing `CONFIG_DIR`")
    } else {
        dirs::config_dir()
            .expect("Could not fetch the user's configuration directory")
            .join(env!("CARGO_PKG_NAME"))
    };

    fs::create_dir_all(&config_dir).expect("Error creating directory");

    let config_path = config_dir.join("config.json");
    config_path.into_os_string().into_string().unwrap()
}

/// Represents the configuration file
#[derive(Args, Builder, Clone, Debug, Deserialize, Serialize, PartialEq)]
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
    is_default: bool,
    #[serde(skip)]
    is_loaded: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plex_url: "http://127.0.0.1:32400".to_string(),
            plex_token: "PLEX_TOKEN".to_string(),
            primary_section_id: 0,
            profiles_directory: "./profiles".to_string(),
            is_default: true,
            is_loaded: false,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn set_profiles_directory(&mut self, dir: &str) {
        self.profiles_directory = dir.to_string()
    }

    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    pub async fn save_config(&self, config_path: Option<&str>) -> Result<()> {
        debug!("Saving config to disk...");

        let default_config_path = build_config_path();
        let config_path = if let Some(config_path) = config_path {
            fs::create_dir_all(config_path)?;
            Path::new(config_path).join("config.json")
        } else {
            Path::new(&default_config_path).to_path_buf()
        };
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(config_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}

pub async fn load_config() -> Result<Config> {
    debug!("Loading config...");

    let config_path = if let Ok(dir) = env::var("CONFIG_DIR") {
        Path::new(&dir).join("config.json")
    } else {
        Path::new(&build_config_path()).to_path_buf()
    };
    debug!("{}", &config_path.display());

    if !config_path.exists() {
        return build_config_wizard().await;
    }

    let mut file = File::open(config_path)?;
    let mut config = String::default();
    file.read_to_string(&mut config)?;

    if let Ok(mut config) = serde_json::from_str::<Config>(&config) {
        config.is_loaded = true;
        Ok(config)
    } else {
        Ok(build_config_wizard().await?)
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
        let plex_url = PlexUrl::try_new(plex_url)?;

        let plex_token = if let Ok(plex_token) = env::var("PLEX_TOKEN") {
            plex_token
        } else {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your plex token:")
                .interact_text()?
                .to_string()
        };
        let plex_token = PlexToken::try_new(plex_token)?;

        info!("Testing connection to plex. Please wait...");
        if PlexClient::new_for_config(&plex_url, &plex_token)
            .await
            .is_ok()
        {
            info!("Success!");
            break (plex_url, plex_token);
        } else {
            error!("Could not connect to plex. Please re-enter your URL and token.")
        }
    };

    let primary_section_id = if let Ok(id) = env::var("PRIMARY_SECTION_ID") {
        id.parse::<i32>()
    } else {
        let plex = PlexClient::new_for_config(&plex_url, &plex_token).await?;
        let sections = plex.get_music_sections();
        let titles = sections
            .iter()
            .map(|x| x.get_title().to_owned())
            .collect::<Vec<String>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your music library:")
            .default(0)
            .items(&titles)
            .interact()?;
        sections[selection].id().parse::<i32>()
    }
    .expect("Could not parse section id");

    let config = ConfigBuilder::default()
        .profiles_directory(profiles_directory)
        .plex_url(plex_url.to_string())
        .plex_token(plex_token.to_string())
        .primary_section_id(primary_section_id)
        .is_default(false)
        .is_loaded(true)
        .build()?;
    let data = serde_json::to_string_pretty(&config)?;

    let mut file = File::create(build_config_path())?;
    file.write_all(data.as_bytes())?;

    Ok(config)
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::default();
        output += &format!("Plex URL:       {}\n", self.get_plex_url());
        output += &format!("Default Config: {}\n", self.is_default);

        write!(f, "{}", output)
    }
}
