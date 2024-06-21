//! Profile wizards

use anyhow::{anyhow, Context, Result};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use simplelog::info;
use strum::VariantNames;

use crate::profiles::profile::{Profile, ProfileBuilder};
use crate::profiles::profile_section::{ProfileSection, ProfileSectionBuilder, Sections};
use crate::profiles::types::{ProfileSourceId, ProfileTitle, RefreshInterval};
use crate::profiles::{ProfileSource, SectionType, VALID_INTERVALS};
use crate::state::AppState;

/// The main entrypoint of the wizard
pub async fn create_profile_wizard(app_state: &AppState) -> Result<Profile> {
    let profile_name = set_profile_name(app_state).await?;

    let summary = set_summary()?;
    let refresh_interval = select_refresh_interval()?;
    let time_limit = set_time_limit()?;

    let profile_source = select_profile_source()?;
    let profile_source_id = select_profile_source_id(profile_source, app_state).await?;

    let sections = select_profile_sections()?;

    let profile = ProfileBuilder::default()
        .title(profile_name)
        .summary(summary)
        .profile_source(profile_source)
        .profile_source_id(profile_source_id)
        .sections(sections)
        .refresh_interval(refresh_interval)
        .time_limit(time_limit)
        .build()?;

    Ok(profile)
}

async fn set_profile_name(app_state: &AppState) -> Result<ProfileTitle> {
    let profile_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("What is the name of your new profile? This will also be the name of the playlist on the plex server.")
        .interact_text()?;

    if app_state.get_profile_by_title(&profile_name).is_some() {
        let choice = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Profile `{profile_name}` already exists. Do you want to overwrite this profile?"
            ))
            .default(false)
            .interact()?;

        if !choice {
            return Err(anyhow!("Profile already exists"));
        }
    }

    if app_state.get_playlist_by_title(&profile_name).is_some() {
        let choice = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Playlist `{profile_name}` already exists in plex. Do you want to overwrite this playlist?"))
            .default(false)
            .interact()?;

        if !choice {
            return Err(anyhow!("Playlist already exists in plex"));
        }
    }

    Ok(ProfileTitle::new(profile_name)
        .with_context(|| "Error setting profile title from wizard")?)
}

fn set_summary() -> Result<String> {
    let summary = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("What is the summary for your new profile? This will also be the summary of the playlist on the plex server.")
        .default(String::default())
        .interact_text()?;

    Ok(summary)
}

fn select_refresh_interval() -> Result<RefreshInterval> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the refresh interval for this profile:")
        .default(0)
        .items(&VALID_INTERVALS.map(|i| format!("{i} minutes")))
        .interact()?;

    Ok(RefreshInterval::new(VALID_INTERVALS[selection])?)
}

fn set_time_limit() -> Result<u32> {
    let time_limit = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a time limit in hours for the profile, or `0` for no time limit:")
        .default("24".to_string())
        .interact_text()?
        .parse::<u32>()?;

    Ok(time_limit)
}

fn select_profile_source() -> Result<ProfileSource> {
    let choices = ProfileSource::VARIANTS;
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the source for this profile:")
        .default(0)
        .items(choices)
        .interact()?;

    Ok(ProfileSource::from_repr(selection).unwrap())
}

async fn select_profile_source_id(
    profile_source: ProfileSource,
    app_state: &AppState,
) -> Result<Option<ProfileSourceId>> {
    let plex = app_state.get_plex_client();

    let id = match profile_source {
        ProfileSource::Library => None,
        ProfileSource::Collection => {
            let collections = plex.get_collections();
            let titles = collections
                .iter()
                .map(|x| x.title.as_str())
                .collect::<Vec<&str>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a collection:")
                .default(0)
                .items(&titles)
                .interact()?;

            Some(collections[selection].rating_key.to_owned())
        }
        ProfileSource::Playlist => {
            let playlists = plex.get_playlists();
            let titles = playlists
                .iter()
                .map(|x| x.title.as_str())
                .collect::<Vec<&str>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a playlist")
                .default(0)
                .items(&titles)
                .interact()?;

            Some(playlists[selection].rating_key.to_owned())
        }
        ProfileSource::SingleArtist => {
            let artist: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Search for an artist:")
                .interact_text()?;

            info!("Searching for artists. Please wait...");
            let artists = plex.search_for_artist(&artist).await?;

            let names = &artists
                .iter()
                .map(|x| x.title.to_owned())
                .collect::<Vec<String>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select an artist:")
                .default(0)
                .items(names)
                .interact()?;

            Some(artists[selection].id().to_owned())
        }
    };

    Ok(match id {
        Some(id) => Some(ProfileSourceId::new(id)?),
        None => None,
    })
}

fn select_profile_sections() -> Result<Sections> {
    let defaults = &[false, false, false];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Which sections do you want to include in your profile?")
        .items(SectionType::VARIANTS)
        .defaults(defaults)
        .interact()?;

    let selections = if selections.is_empty() {
        vec![0, 1, 2]
    } else {
        selections
    };

    let mut sections = Sections::default();

    if selections.contains(&0) {
        sections.set_unplayed_tracks(build_profile_section(SectionType::Unplayed)?)
    }

    if selections.contains(&1) {
        sections.set_least_played_tracks(build_profile_section(SectionType::LeastPlayed)?)
    }

    if selections.contains(&2) {
        sections.set_oldest_tracks(build_profile_section(SectionType::Oldest)?)
    }

    Ok(sections)
}

fn build_profile_section(section_type: SectionType) -> Result<ProfileSection> {
    println!("\nBuilding Section: {section_type}");

    let deduplicate_tracks_by_guid = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to deduplicate tracks by their Plex GUID?")
        .default(true)
        .interact()?;

    let deduplicate_by_track_and_artist = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to deduplicate tracks with the same title and artist?")
        .default(true)
        .interact()?;

    let maximum_tracks_by_artists =
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter a maximum number of tracks that can appear in a playlist by a single artist. (A value of `0` disables any limit.)")
            .default(25)
            .validate_with(|input: &i32| -> Result<(), &str> {
                if *input >= 0 {
                    Ok(())
                } else {
                    Err("Value cannot be less than zero")
                }
            })
            .interact_text()?;

    let minimum_track_rating = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a minimum star rating for included tracks:")
        .default(3)
        .validate_with(|input: &u32| -> Result<(), &str> {
            if *input <= 5 {
                Ok(())
            } else {
                Err("Minimum rating cannot be greater than five")
            }
        })
        .interact_text()?;

    let randomize = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to randomize the track order?")
        .default(true)
        .interact()?;

    // TODO get valid sort fields from plex
    let sorting = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a comma separated list of fields to sort")
        .default(get_default_sorting(section_type))
        // TODO validate
        .interact_text()?;

    let section = ProfileSectionBuilder::default()
        .enabled(true)
        .section_type(section_type)
        .deduplicate_tracks_by_guid(deduplicate_tracks_by_guid)
        .deduplicate_tracks_by_title_and_artist(deduplicate_by_track_and_artist)
        .maximum_tracks_by_artist(maximum_tracks_by_artists)
        .minimum_track_rating(minimum_track_rating)
        .randomize_tracks(randomize)
        .sorting(sorting)
        .build()?;

    Ok(section)
}

fn get_default_sorting(section_type: SectionType) -> String {
    match section_type {
        SectionType::Unplayed => vec![
            "userRating:desc",
            "viewCount",
            "lastViewedAt",
            "guid",
            "mediaBitrate:desc",
        ],
        SectionType::LeastPlayed => vec!["viewCount", "lastViewedAt", "guid", "mediaBitrate:desc"],
        SectionType::Oldest => vec!["lastViewedAt", "viewCount", "guid", "mediaBitrate:desc"],
    }
    .join(",")
}
