use anyhow::Result;
use clap::Args;
use simplelog::info;

use crate::profiles::{profile, ProfileAction};
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
            let mut profile = profile::create_profile_wizard(app_state).await?;
            Profile::build_playlist(&mut profile, ProfileAction::Create, app_state.get_plex()).await?;
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
