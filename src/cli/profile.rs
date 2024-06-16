use anyhow::Result;
use clap::Args;
use simplelog::info;

use crate::profiles::{ProfileAction, wizards};
use crate::profiles::profile::Profile;
use crate::state::AppState;

#[derive(Args, Debug)]
pub struct CliProfile {
    #[command(subcommand)]
    pub profile_cmds: ProfileAction,
}

pub async fn run_profile_command(profile: CliProfile, app_state: &AppState) -> Result<()> {
    match profile.profile_cmds {
        ProfileAction::Create => {
            let mut profile = wizards::create_profile_wizard(app_state).await?;
            Profile::build_playlist(&mut profile, app_state, ProfileAction::Create).await?;
            profile.save_to_file(app_state.get_profiles_directory()).await?;

            info!("Profile created successfully!")
        }
        ProfileAction::Edit => {}
        ProfileAction::Delete => {}
        ProfileAction::List => app_state.list_profiles(),
        ProfileAction::Update => {}
        ProfileAction::View => {}
    }

    Ok(())
}
