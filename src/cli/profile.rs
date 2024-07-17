use anyhow::Result;
use clap::Args;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use simplelog::{debug, info};

use crate::db;
use crate::profiles::manager::ProfileManager;
use crate::profiles::profile::Profile;
use crate::profiles::{wizards, ProfileAction};

#[derive(Args, Debug, PartialEq)]
pub struct CliProfile {
    #[command(subcommand)]
    pub profile_cmds: ProfileAction,
}

pub async fn run_profile_command(profile: CliProfile, mut manager: ProfileManager) -> Result<()> {
    match profile.profile_cmds {
        ProfileAction::Create => {
            let (profile, sections) = wizards::create_profile_wizard(&manager).await?;
            manager.create_playlist(&profile, &sections).await?;
            // db::profiles::create_profile(&profile, &sections).await?;

            info!("Profile created successfully!")
        }
        ProfileAction::Edit => {}
        ProfileAction::Delete => {}
        ProfileAction::List => manager.list_profiles_and_sections().await?,
        ProfileAction::Preview => {
            preview_playlist(&manager).await?;
        }
        ProfileAction::Update => {}
        ProfileAction::View => view_playlist(&manager).await?,
    }

    Ok(())
}

async fn preview_playlist(manager: &ProfileManager) -> Result<()> {
    if !manager.have_profiles().await? {
        println!("No profiles found.");
        return Ok(());
    }

    let profile = select_profile("Select which profile you would like to preview:").await?;
    manager.preview_playlist(&profile).await?;

    Ok(())
}

async fn view_playlist(manager: &ProfileManager) -> Result<()> {
    if !manager.have_profiles().await? {
        println!("No profiles found.");
        return Ok(());
    }

    let profile = select_profile("Select which profile you would like to view:").await?;
    println!("{profile}");

    // Print raw json of profile
    debug!("{}\n", serde_json::to_string_pretty(&profile).unwrap());
    Ok(())
}

async fn select_profile(prompt: &str) -> Result<Profile> {
    let titles = db::profiles::fetch_profile_titles().await?;
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&titles)
        .default(0)
        .interact()?;

    let profile = db::profiles::fetch_profile_by_title(&titles[selection])
        .await?
        .unwrap();

    Ok(profile)
}
