use anyhow::Result;
use clap::Args;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use simplelog::{debug, info};

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
            let mut profile = wizards::create_profile_wizard().await?;
            Profile::build_playlist(&mut profile, ProfileAction::Create, None).await?;
            profile
                .save_to_file(APP_STATE.get().read().await.get_profiles_directory()?)
                .await?;

            info!("Profile created successfully!")
        }
        ProfileAction::Edit => {}
        ProfileAction::Delete => {}
        ProfileAction::List => APP_STATE.get().read().await.list_profiles(),
        ProfileAction::Preview => {
            preview_playlist().await?;
        }
        ProfileAction::Update => {}
        ProfileAction::View => view_playlist().await?,
    }

    Ok(())
}

async fn preview_playlist() -> Result<()> {
    let app_state = APP_STATE.get().read().await;

    if !app_state.have_profiles() {
        println!("No profiles found.");
        return Ok(());
    }

    let profile = select_profile("Select which profile you would like to preview:").await?;
    Profile::build_playlist(&mut profile.clone(), ProfileAction::Preview, None).await?;

    Ok(())
}

async fn view_playlist() -> Result<()> {
    if !APP_STATE.get().read().await.have_profiles() {
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
    let app_state = APP_STATE.get().read().await;

    let titles = app_state.get_profile_titles();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&titles)
        .default(0)
        .interact()?;

    let profile = app_state
        .get_profile_by_title(titles[selection])
        .unwrap()
        .to_owned();

    Ok(profile)
}
