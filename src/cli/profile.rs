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
            let profile = wizards::create_profile_wizard(&manager).await?;
            let new_profile_key = manager.add_new_profile(&profile);
            let merger = manager.fetch_profile_tracks(new_profile_key, None).await?;
            manager
                .create_playlist(&profile, new_profile_key, &merger)
                .await?;
            db::profiles::create_profile(&profile).await?;

            info!("Profile created successfully!")
        }
        ProfileAction::Edit => {}
        ProfileAction::Delete => {}
        ProfileAction::List => manager.list_profiles(),
        ProfileAction::Preview => {
            preview_playlist(&manager).await?;
        }
        ProfileAction::Update => {}
        ProfileAction::View => view_playlist(&manager).await?,
    }

    Ok(())
}

async fn preview_playlist(manager: &ProfileManager) -> Result<()> {
    if !manager.have_profiles() {
        println!("No profiles found.");
        return Ok(());
    }

    let profile =
        select_profile("Select which profile you would like to preview:", manager).await?;
    manager.preview_playlist(&profile).await?;

    Ok(())
}

async fn view_playlist(manager: &ProfileManager) -> Result<()> {
    if !manager.have_profiles() {
        println!("No profiles found.");
        return Ok(());
    }

    let profile = select_profile("Select which profile you would like to view:", manager).await?;
    println!("{profile}");

    // Print raw json of profile
    debug!("{}\n", serde_json::to_string_pretty(&profile).unwrap());
    Ok(())
}

async fn select_profile(prompt: &str, manager: &ProfileManager) -> Result<Profile> {
    let titles = manager.get_profile_titles();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&titles)
        .default(0)
        .interact()?;

    let profile = manager
        .get_profile_by_title(&titles[selection])
        .unwrap()
        .to_owned();

    Ok(profile)
}
