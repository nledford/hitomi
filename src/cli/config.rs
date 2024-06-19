use anyhow::Result;
use clap::{Args, Subcommand};

use crate::config as AppConfig;
use crate::config::ConfigBuilder as AppConfigBuilder;

#[derive(Args, PartialEq)]
pub struct CliConfig {
    #[command(subcommand)]
    config_cmds: ConfigCmds,
}

#[derive(Subcommand, PartialEq)]
enum ConfigCmds {
    Create(CreateArgs),
    Update(UpdateArgs),
    View,
}

#[derive(Args, PartialEq)]
struct CreateArgs {
    #[arg(long)]
    config_directory: String,
    #[arg(long)]
    plex_url: String,
    #[arg(long)]
    plex_token: String,
    #[arg(long)]
    profiles_directory: String,
    #[arg(long)]
    primary_section_id: i32,
}

#[derive(Args, PartialEq)]
struct UpdateArgs {
    #[arg(long)]
    profiles_directory: Option<String>,
}

pub async fn run_config_cmd(cfg: CliConfig) -> Result<()> {
    match cfg.config_cmds {
        ConfigCmds::Create(cmd) => {
            let new_config = AppConfigBuilder::default()
                .plex_token(cmd.plex_token)
                .plex_url(cmd.plex_url)
                .profiles_directory(cmd.profiles_directory)
                .primary_section_id(cmd.primary_section_id)
                .build()?;

            new_config.save_config(Some(&cmd.config_directory)).await?;
        }
        ConfigCmds::View => {
            let _config = AppConfig::load_config().await;
            // config.print_table();
        }
        ConfigCmds::Update(args) => {
            let mut config = AppConfig::load_config().await?;

            if let Some(profiles_directory) = args.profiles_directory {
                config.set_profiles_directory(&profiles_directory);
            }

            config.save_config(None).await?;
            // config.print_table();
        }
    }

    Ok(())
}
