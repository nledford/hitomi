use anyhow::Result;
use clap::Args;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use simplelog::info;

use crate::profiles::profile::Profile;
use crate::profiles::{wizards, ProfileAction};
use crate::state::AppState;

#[derive(Args, Debug, PartialEq)]
pub struct CliProfile {
    #[command(subcommand)]
    pub profile_cmds: ProfileAction,
}

pub async fn run_profile_command(profile: CliProfile, app_state: &AppState) -> Result<()> {
    match profile.profile_cmds {
        ProfileAction::Create => {
            let mut profile = wizards::create_profile_wizard(app_state).await?;
            Profile::build_playlist(&mut profile, app_state, ProfileAction::Create, None).await?;
            profile
                .save_to_file(app_state.get_profiles_directory())
                .await?;

            info!("Profile created successfully!")
        }
        ProfileAction::Edit => {}
        ProfileAction::Delete => {}
        ProfileAction::List => app_state.list_profiles(),
        ProfileAction::Preview => {
            preview_playlist(app_state).await?;
        }
        ProfileAction::Update => {}
        ProfileAction::View => view_playlist(app_state)?,
    }

    Ok(())
}

async fn preview_playlist(app_state: &AppState) -> Result<()> {
    if !app_state.have_profiles() {
        println!("No profiles found.");
        return Ok(());
    }

    let profile = select_profile("Select which profile you would like to preview:", app_state)?;
    Profile::build_playlist(
        &mut profile.clone(),
        app_state,
        ProfileAction::Preview,
        None,
    )
    .await?;

    Ok(())
}

fn view_playlist(app_state: &AppState) -> Result<()> {
    if !app_state.have_profiles() {
        println!("No profiles found.");
        return Ok(());
    }

    let profile = select_profile("Select which profile you would like to view:", app_state)?;
    println!("{profile}");
    Ok(())
}

fn select_profile<'a>(prompt: &'a str, app_state: &'a AppState) -> Result<&'a Profile> {
    let titles = app_state.get_profile_titles();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&titles)
        .default(0)
        .interact()?;

    let profile = app_state.get_profile_by_title(titles[selection]).unwrap();

    Ok(profile)
}
