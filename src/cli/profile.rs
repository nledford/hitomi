use anyhow::Result;
use clap::Args;
use simplelog::info;

use crate::profiles::profile::Profile;
use crate::profiles::{profile, ProfileAction};
use crate::state::APP_STATE;

#[derive(Args, Debug)]
pub struct CliProfile {
    #[command(subcommand)]
    pub profile_cmds: ProfileAction,
}

pub async fn run_profile_command(profile: CliProfile) -> Result<()> {
    match profile.profile_cmds {
        ProfileAction::Create => {
            let mut profile = profile::create_profile_wizard().await?;
            Profile::build_playlist(&mut profile, ProfileAction::Create).await?;
            profile.save_to_file().await?;

            info!("Profile created successfully!")
        }
        ProfileAction::Edit => {}
        ProfileAction::Delete => {}
        ProfileAction::List => APP_STATE.lock().await.list_profiles(),
        ProfileAction::Update => {}
        ProfileAction::View => {}
    }

    Ok(())
}
