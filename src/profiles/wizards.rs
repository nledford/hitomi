//! Profile wizards

use crate::plex;
use crate::profiles::profile::{Profile, ProfileBuilder};
use crate::profiles::profile_section::{ProfileSection, ProfileSectionBuilder};
use crate::profiles::types::{ProfileSectionSort, ProfileSourceId, RefreshInterval};
use crate::profiles::{ProfileSource, SectionType, VALID_INTERVALS};
use crate::state::APP_STATE;
use crate::types::Title;
use anyhow::{anyhow, Context, Result};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use simplelog::info;
use strum::VariantNames;

/// The main entrypoint of the wizard
pub async fn create_profile_wizard() -> Result<Profile> {
    let profile_name = set_profile_name().await?;

    let summary = set_summary()?;
    let refresh_interval = select_refresh_interval()?;
    let time_limit = set_time_limit()?;

    let profile_source = select_profile_source()?;
    let profile_source_id = select_profile_source_id(profile_source).await?;

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

async fn set_profile_name() -> Result<Title> {
    let profile_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("What is the name of your new profile? This will also be the name of the playlist on the plex server.")
        .interact_text()?;
    let title = Title::try_new(profile_name.clone())
        .with_context(|| "Error setting profile/playlist title from wizard")?;

    let app_state = APP_STATE.get().read().await;
    if app_state
        .get_profile_manager()
        .get_profile_by_title(&title)
        .is_some()
    {
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

    if app_state.get_playlist_by_title(&title).is_some() {
        let choice = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Playlist `{profile_name}` already exists in plex. Do you want to overwrite this playlist?"))
            .default(false)
            .interact()?;

        if !choice {
            return Err(anyhow!("Playlist already exists in plex"));
        }
    }

    Ok(title)
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

    Ok(RefreshInterval::try_new(VALID_INTERVALS[selection])?)
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
) -> Result<Option<ProfileSourceId>> {
    let plex = plex::get_plex_client().await;

    let id: Option<String> = match profile_source {
        ProfileSource::Library => None,
        ProfileSource::Collection => {
            let collections = plex.get_collections();
            let titles = collections
                .iter()
                .map(|x| x.get_title())
                .collect::<Vec<&str>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a collection:")
                .default(0)
                .items(&titles)
                .interact()?;

            let id = collections[selection].get_id().to_owned();

            Some(id)
        }
        ProfileSource::Playlist => {
            let playlists = plex.get_playlists();
            let titles = playlists
                .iter()
                .map(|x| x.get_title())
                .collect::<Vec<&str>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a playlist")
                .default(0)
                .items(&titles)
                .interact()?;

            Some(playlists[selection].get_id().to_owned())
        }
        ProfileSource::SingleArtist => {
            let artist: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Search for an artist:")
                .interact_text()?;

            info!("Searching for artists. Please wait...");
            let artists = plex.search_for_artist(&artist).await?;

            let names = &artists
                .iter()
                .map(|x| x.get_title().to_owned())
                .collect::<Vec<_>>();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select an artist:")
                .default(0)
                .items(names)
                .interact()?;

            let id = artists[selection].get_id().to_owned();

            Some(id)
        }
    };

    Ok(match id {
        Some(id) => Some(ProfileSourceId::try_new(id)?),
        None => None,
    })
}

fn select_profile_sections() -> Result<Vec<ProfileSection>> {
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

    let mut sections = vec![];

    if selections.contains(&0) {
        sections.push(build_profile_section(SectionType::Unplayed)?)
    }

    if selections.contains(&1) {
        sections.push(build_profile_section(SectionType::LeastPlayed)?)
    }

    if selections.contains(&2) {
        sections.push(build_profile_section(SectionType::Oldest)?)
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
    let section_sort = ProfileSectionSort::default_from(section_type);
    let sorting = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a comma separated list of fields to sort")
        .default(section_sort.into_inner())
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
