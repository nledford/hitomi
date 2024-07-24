//! Manages profiles

use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use chrono::{Local, Timelike, Utc};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use itertools::Itertools;
use simplelog::{error, info};
use tokio::task::JoinSet;

use crate::plex::models::playlists::Playlist;
use crate::plex::models::tracks::Track;
use crate::plex::types::PlexId;
use crate::plex::PlexClient;
use crate::profiles::profile::Profile;
use crate::profiles::profile_section::ProfileSection;
use crate::profiles::profile_tracks::ProfileTracks;
use crate::profiles::refresh_result::RefreshResult;
use crate::profiles::ProfileAction;
use crate::{config, db};

#[derive(Clone, Debug, Default)]
pub struct ProfileManager {
    plex_client: PlexClient,
    playlists: Vec<Playlist>,
}

// INITIALIZATION
impl ProfileManager {
    pub async fn new() -> Result<Self> {
        let config = config::load_config().await?;
        let plex_client = PlexClient::initialize(&config).await?;
        let playlists = plex_client.get_playlists().to_vec();

        let manager = ProfileManager {
            plex_client,
            playlists,
        };
        Ok(manager)
    }
}

// PlEX
impl ProfileManager {
    pub fn get_plex_client(&self) -> &PlexClient {
        &self.plex_client
    }
}

// PLAYLISTS
impl ProfileManager {
    pub fn get_playlist_by_title(&self, title: &str) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.get_title() == title)
    }
}

impl ProfileManager {
    pub async fn have_profiles(&self) -> Result<bool> {
        Ok(!db::profiles::fetch_profiles(true).await?.is_empty())
    }

    pub async fn get_profiles_to_refresh(&self, ran_once: bool) -> Result<Vec<Profile>> {
        if ran_once && !self.fetch_any_profile_refresh().await? {
            return Ok(vec![]);
        }
        let to_refresh = db::profiles::fetch_profiles_to_refresh(!ran_once).await?;
        Ok(to_refresh)
    }

    pub async fn list_profiles_and_sections(&self) -> Result<()> {
        let profiles = db::profiles::fetch_profiles(false).await?;

        for profile in profiles {
            println!("{}", profile.get_title());

            for section in profile.fetch_sections().await? {
                println!(" - {}", section.get_section_type())
            }
        }

        Ok(())
    }

    pub async fn fetch_any_profile_refresh(&self) -> Result<bool> {
        if Utc::now().second() != 0 {
            return Ok(false);
        }

        let any = db::profiles::fetch_any_eligible_for_refresh().await?;
        Ok(any)
    }

    async fn print_update(&self) -> Result<()> {
        let profiles = db::profiles::fetch_profiles(true).await?;
        let str = profiles
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc: HashMap<String, Vec<String>>, profile| {
                    acc.entry(profile.get_next_refresh_hour_minute())
                        .or_default()
                        .push(profile.get_title().to_owned());
                    acc
                },
            )
            .into_iter()
            .sorted()
            .fold(String::default(), |mut acc, (k, v)| {
                acc += &format!("  <b>Refreshing at {k}:</b>\n");
                for title in v.iter().sorted() {
                    acc += &format!("    - {title}\n");
                }
                acc
            });
        info!("Upcoming refreshes:\n{str}");

        Ok(())
    }

    pub async fn refresh_playlists_from_profiles(
        &self,
        run_loop: bool,
        ran_once: bool,
    ) -> Result<()> {
        if ran_once && !self.fetch_any_profile_refresh().await? {
            return Ok(());
        }

        let profiles = self.get_profiles_to_refresh(ran_once).await?;
        let mut set = JoinSet::new();
        for profile in profiles {
            set.spawn(update_playlist(self.get_plex_client().to_owned(), profile));
        }

        let mut results = vec![];
        while let Some(res) = set.join_next().await {
            let res = res?;

            match res {
                Ok(refresh_result) => results.push(refresh_result),
                Err(err) => {
                    error!("An error occurred while attempting to refresh playlists`: {err}")
                }
            }
        }

        info!(
            "<b>{} Profile{} updated at {}:</b>",
            results.len(),
            if results.len() == 1 { "" } else { "s" },
            Local::now().format("%T")
        );
        for result in results.iter().sorted_by_key(|result| result.get_title()) {
            println!("{result}\n");
        }

        if run_loop {
            self.print_update().await?;
        }

        Ok(())
    }

    pub async fn create_playlist(
        &mut self,
        profile: &Profile,
        sections: &[ProfileSection],
    ) -> Result<()> {
        let save = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to save this profile?")
            .default(true)
            .interact()?;

        if save {
            info!("Creating playlist in plex...");
            let playlist_id = self.plex_client.create_playlist(profile).await?;
            let playlist_id = PlexId::try_new(playlist_id)?;

            info!("Saving profile to database...");
            db::profiles::create_profile(playlist_id.as_str(), profile, sections).await?;

            info!("Adding tracks to newly created playlist...");
            let profile_tracks = ProfileTracks::new(self.get_plex_client(), profile).await?;
            self.plex_client
                .add_items_to_playlist(&playlist_id, &profile_tracks.get_track_ids())
                .await?;

            print_refresh_results(
                profile_tracks.get_merged_tracks(),
                profile.get_title(),
                ProfileAction::Create,
            );
        } else {
            info!("Playlist not saved");
        }

        Ok(())
    }

    pub async fn preview_playlist(&self, profile: &Profile) -> Result<()> {
        let profile_tracks = ProfileTracks::new(self.get_plex_client(), profile).await?;
        profile_tracks.print_preview();

        Ok(())
    }
}

// UTILITY FUNCTIONS #############################################################

fn print_refresh_results(tracks: &[Track], playlist_title: &str, action: ProfileAction) {
    let size = tracks.len();

    let duration: i64 = tracks.iter().map(|t| t.get_track_duration()).sum();
    let duration = Duration::from_millis(duration as u64);
    let duration = humantime::format_duration(duration).to_string();

    let action = if action == ProfileAction::Create {
        "created"
    } else {
        "updated"
    };

    log::info!(
        "Successfully {} `{}` playlist!\n\tFinal size: {}\n\tFinal duration: {}",
        action,
        playlist_title,
        size,
        duration
    );
}

async fn update_playlist(plex_client: PlexClient, profile: Profile) -> Result<RefreshResult> {
    let profile_tracks = ProfileTracks::new(&plex_client, &profile).await?;
    info!("Updating `{}` playlist...", profile.get_title());

    info!("Wiping destination playlist...");
    plex_client
        .clear_playlist(profile.get_playlist_id())
        .await?;

    info!("Updating destination playlist...");
    plex_client
        .add_items_to_playlist(profile.get_playlist_id(), &profile_tracks.get_track_ids())
        .await?;

    let summary = format!(
        "{}\n{}",
        profile.get_next_refresh_str(),
        profile.get_summary()
    );
    plex_client
        .update_summary(profile.get_playlist_id(), &summary)
        .await?;

    let refresh_result = RefreshResult::new(
        profile.get_title(),
        profile_tracks.get_merged_tracks(),
        ProfileAction::Update,
    );

    Ok(refresh_result)
}
