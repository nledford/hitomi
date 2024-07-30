//! Configuration for `hitomi`

use std::env;
use std::fmt::{Display, Formatter};

use anyhow::Result;
use clap::Args;
use derive_builder::Builder;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use simplelog::{debug, info};

use crate::db;
use crate::plex::PlexClient;
use crate::types::plex::plex_token::PlexToken;

/// Represents the configuration file
#[derive(Args, Builder, Clone, Debug, Deserialize, Serialize, PartialEq, sqlx::Type)]
pub struct Config {
    #[arg(long)]
    plex_token: String,
    #[arg(long)]
    plex_url: String,
    #[arg(long)]
    primary_section_id: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plex_url: "http://127.0.0.1:32400".to_string(),
            plex_token: "PLEX_TOKEN".to_string(),
            primary_section_id: 0,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_plex_url(&self) -> Result<Url> {
        Ok(Url::parse(&self.plex_url)?)
    }

    pub fn get_plex_url_str(&self) -> String {
        self.get_plex_url().unwrap().to_string()
    }

    pub fn get_plex_token(&self) -> Result<PlexToken> {
        Ok(PlexToken::try_new(&self.plex_token)?)
    }

    pub fn get_primary_section_id(&self) -> u32 {
        self.primary_section_id
    }
}

/// Wizard used by user to create an initial configuration table
pub async fn build_config_wizard() -> Result<Config> {
    info!("Config table not populated. Checking for environment variables...");

    let plex_url = if let Ok(plex_url) = env::var("PLEX_URL") {
        plex_url
    } else {
        Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your plex URL:")
            .interact_text()?
            .to_string()
    };
    let plex_url = Url::parse(&plex_url)?;

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
    match PlexClient::new_for_config(&plex_url, &plex_token).await {
        Ok(_) => {
            info!("Successfully connected to plex!");
        }
        Err(err) => {
            panic!("Could not connect to plex:\n{err}")
        }
    };

    let primary_section_id = if let Ok(id) = env::var("PRIMARY_SECTION_ID") {
        id.parse::<u32>()
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
        sections[selection].id().parse::<u32>()
    }
    .expect("Could not parse section id");

    let config = ConfigBuilder::default()
        .plex_url(plex_url.to_string())
        .plex_token(plex_token.to_string())
        .primary_section_id(primary_section_id)
        .build()?;

    db::config::save_config(&config).await?;

    Ok(config)
}

pub async fn load_config() -> Result<Config> {
    debug!("Loading config...");

    if !db::config::have_config().await? {
        info!("Config not found in database.");
        return build_config_wizard().await;
    }

    let config = db::config::fetch_config().await?;

    Ok(config)
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::default();
        output += &format!("Plex URL:       {}\n", self.get_plex_url_str());

        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const VALID_TOKEN: &str = "RWtuIcHBY-hq6HbSq3GY";
    const VALID_URL: &str = "http://127.0.0.1:32400";

    #[test]
    fn test_valid_config() {
        let config = ConfigBuilder::default()
            .plex_token(VALID_TOKEN.to_string())
            .plex_url(VALID_URL.to_string())
            .primary_section_id(1)
            .build()
            .unwrap();

        let valid_token = PlexToken::try_new(VALID_TOKEN).unwrap();
        assert_eq!(config.get_plex_token().unwrap(), valid_token);

        let valid_url = Url::parse(VALID_URL).unwrap();
        assert_eq!(config.get_plex_url().unwrap(), valid_url);
    }

    #[test]
    #[should_panic]
    fn test_invalid_config_token() {
        let config = ConfigBuilder::default()
            .plex_token("rucpkuXGIn/1ZlqJPBVaYZQduMJWX5yWGQan20nOpFokXbGviXonA==".to_string())
            .plex_url(VALID_URL.to_string())
            .primary_section_id(1)
            .build()
            .unwrap();

        config.get_plex_token().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_config_url() {
        let config = ConfigBuilder::default()
            .plex_token(VALID_TOKEN.to_string())
            .plex_url("It dawned on her that others could make her happier, but only she could make herself happy.".to_string())
            .primary_section_id(1)
            .build()
            .unwrap();

        config.get_plex_url().unwrap();
    }
}
