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

/// Represents the configuration file
#[derive(Args, Builder, Clone, Debug, Deserialize, Serialize, PartialEq, sqlx::Type)]
pub struct Config {
    #[arg(long)]
    plex_token: String,
    #[arg(long)]
    plex_url: String,
    #[arg(long)]
    primary_section_id: i32,
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

    pub fn get_plex_url(&self) -> &str {
        &self.plex_url
    }

    pub fn get_plex_token(&self) -> &str {
        &self.plex_token
    }

    pub fn get_primary_section_id(&self) -> i32 {
        self.primary_section_id
    }
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
        .plex_url(plex_url.to_string())
        .plex_token(plex_token.to_string())
        .primary_section_id(primary_section_id)
        .build()?;
    let data = serde_json::to_string_pretty(&config)?;

    // TODO save config to database

    Ok(config)
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::default();
        output += &format!("Plex URL:       {}\n", self.get_plex_url());

        write!(f, "{}", output)
    }
}
