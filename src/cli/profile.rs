use anyhow::Result;
use clap::Args;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use simplelog::{debug, info};

use crate::files;
use crate::profiles::manager::PROFILE_MANAGER;
use crate::profiles::profile::Profile;
use crate::profiles::{wizards, ProfileAction};
use crate::state::APP_STATE;

#[derive(Args, Debug, PartialEq)]
pub struct CliProfile {
    #[command(subcommand)]
    pub profile_cmds: ProfileAction,
}

pub async fn run_profile_command(profile: CliProfile) -> Result<()> {
    match profile.profile_cmds {
        ProfileAction::Create => {
            let profile = wizards::create_profile_wizard().await?;
            {
                let mut manager = PROFILE_MANAGER.get().unwrap().write().await;
                let new_profile_key = manager.add_new_profile(&profile);
                let items = manager.fetch_profile_tracks(new_profile_key, None).await?;
                manager
                    .create_playlist(&profile, new_profile_key, items)
                    .await?;
            }
            files::save_profile_to_disk(&profile).await?;

            info!("Profile created successfully!")
        }
        ProfileAction::Edit => {}
        ProfileAction::Delete => {}
        ProfileAction::List => APP_STATE
            .get()
            .unwrap()
            .read()
            .await
            .get_profile_manager()
            .list_profiles(),
        ProfileAction::Preview => {
            preview_playlist().await?;
        }
        ProfileAction::Update => {}
        ProfileAction::View => view_playlist().await?,
    }

    Ok(())
}

async fn preview_playlist() -> Result<()> {
    let app_state = APP_STATE.get().unwrap().read().await;

    if !app_state.get_profile_manager().have_profiles() {
        println!("No profiles found.");
        return Ok(());
    }

    let profile = select_profile("Select which profile you would like to preview:").await?;
    app_state
        .get_profile_manager()
        .preview_playlist(&profile)
        .await?;

    Ok(())
}

async fn view_playlist() -> Result<()> {
    if !APP_STATE
        .get()
        .unwrap()
        .read()
        .await
        .get_profile_manager()
        .have_profiles()
    {
        println!("No profiles found.");
        return Ok(());
    }

    let profile = select_profile("Select which profile you would like to view:").await?;
    // println!("{profile}");

    // Print raw json of profile
    debug!("{}\n", serde_json::to_string_pretty(&profile).unwrap());
    Ok(())
}

async fn select_profile(prompt: &str) -> Result<Profile> {
    let app_state = APP_STATE.get().unwrap().read().await;

    let titles = app_state.get_profile_manager().get_profile_titles();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&titles)
        .default(0)
        .interact()?;

    let profile = app_state
        .get_profile_manager()
        .get_profile_by_title(&titles[selection])
        .unwrap()
        .to_owned();

    Ok(profile)
}
