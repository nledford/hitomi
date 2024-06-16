use anyhow::Result;
use clap::{Args, Subcommand};

use crate::config::Config;

#[derive(Args, Debug)]
pub struct CliConfig {
    #[command(subcommand)]
    config_cmds: ConfigCmds,
}

#[derive(Debug, Subcommand)]
enum ConfigCmds {
    Read,
    Update(UpdateArgs),
}

#[derive(Args, Debug)]
struct UpdateArgs {
    #[arg(long)]
    profiles_directory: Option<String>,
}

pub async fn run_config_cmd(cfg: CliConfig) -> Result<()> {
    match cfg.config_cmds {
        ConfigCmds::Read => {
            let _config = Config::load_config(None).await;
            // config.print_table();
        }
        ConfigCmds::Update(args) => {
            let mut config = Config::load_config(None).await?;

            if let Some(profiles_directory) = args.profiles_directory {
                config = config.profiles_directory(profiles_directory);
            }

            config.save_config(None).await?;
            // config.print_table();
        }
    }

    Ok(())
}
