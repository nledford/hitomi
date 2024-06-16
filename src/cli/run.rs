use anyhow::Result;
use clap::Args;
use log::info;

use crate::state::AppState;

#[derive(Args, Debug)]
pub struct RunCmds {
    /// Run the application indefinitely, refreshing based on the interval provided in each profile
    #[arg(short = 'l', long, default_value_t = false)]
    pub run_loop: bool,
}

fn print_title(looping: bool) {
    let version = env!("CARGO_PKG_VERSION");

    info!("Plex Playlists v{}", version);

    if looping {
        info!("Application is running in loop mode")
    }
}

pub async fn execute_run_cmd(cmd: RunCmds, app_state: &mut AppState) -> Result<()> {
    print_title(cmd.run_loop);

    app_state.update_state(cmd.run_loop).await?;

    Ok(())
}