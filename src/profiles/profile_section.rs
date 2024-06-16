use std::collections::HashMap;

use anyhow::Result;
use chrono::TimeDelta;
use default_struct_builder::DefaultBuilder;
use nanorand::{ChaCha8, Rng};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::plex::models::Track;
use crate::plex::Plex;
use crate::profiles::{ProfileSource, SectionType};

#[derive(Clone, Debug, Default, DefaultBuilder, Deserialize, PartialEq, Serialize)]
pub struct Sections {
    unplayed_tracks: ProfileSection,
    least_played_tracks: ProfileSection,
    oldest_tracks: ProfileSection,
}

impl Sections {
    pub fn num_enabled(&self) -> i32 {
        let mut count = 0;

        if self.oldest_tracks.enabled {
            count += 1
        }

        if self.least_played_tracks.enabled {
            count += 1
        }

        if self.oldest_tracks.enabled {
            count += 1
        }

        count
    }

    pub async fn fetch_tracks(
        &mut self,
        plex: &Plex,
        profile_source: ProfileSource,
        profile_source_id: &Option<&str>,
        time_limit: f64,
    ) -> Result<()> {
        fetch_section_tracks(
            &mut self.unplayed_tracks,
            plex,
            profile_source,
            profile_source_id,
            time_limit,
        )
        .await?;
        fetch_section_tracks(
            &mut self.least_played_tracks,
            plex,
            profile_source,
            profile_source_id,
            time_limit,
        )
        .await?;
        fetch_section_tracks(
            &mut self.oldest_tracks,
            plex,
            profile_source,
            profile_source_id,
            time_limit,
        )
        .await?;

        Ok(())
    }

    pub fn get_unplayed_tracks(&self) -> Vec<Track> {
        self.unplayed_tracks.tracks.clone()
    }

    fn num_unplayed_tracks(&self) -> usize {
        self.unplayed_tracks.num_tracks()
    }

    pub fn get_least_played_tracks(&self) -> Vec<Track> {
        self.least_played_tracks.tracks.clone()
    }

    fn num_least_played_tracks(&self) -> usize {
        self.least_played_tracks.num_tracks()
    }

    pub fn get_oldest_tracks(&self) -> Vec<Track> {
        self.oldest_tracks.tracks.clone()
    }

    pub fn oldest_tracks_mut(&mut self) -> &mut ProfileSection {
        &mut self.oldest_tracks
    }

    fn num_oldest_tracks(&self) -> usize {
        self.oldest_tracks.num_tracks()
    }

    pub fn global_track_total(&self) -> usize {
        self.num_unplayed_tracks() + self.num_least_played_tracks() + self.num_oldest_tracks()
    }
}

async fn fetch_section_tracks(
    section: &mut ProfileSection,
    plex: &Plex,
    profile_source: ProfileSource,
    profile_source_id: &Option<&str>,
    time_limit: f64,
) -> Result<()> {
    if !section.enabled {
        return Ok(());
    }

    let mut filters = HashMap::new();
    filters.insert(
        "userRating>>".to_string(),
        section.get_minimum_track_rating().to_string(),
    );

    if section.is_unplayed() {
        filters.insert("viewCount".to_string(), "0".to_string());
    } else {
        filters.insert("viewCount>>".to_string(), "0".to_string());
    }

    match profile_source {
        // Nothing special needs to be done for a library source, so this branch is left blank
        ProfileSource::Library => {}
        ProfileSource::Collection => {
            let artists = plex
                .fetch_artists_from_collection(profile_source_id.unwrap())
                .await?;
            let artists = urlencoding::encode(&artists.join(",")).to_string();

            // filters.insert(
            //     "track.userRating>>".to_string(),
            //     section.minimum_track_rating.to_string(),
            // );
            filters.insert("artist.id".to_string(), artists);
        }
        ProfileSource::Playlist => {
            todo!("Playlist option not yet implemented")
        }
        ProfileSource::SingleArtist => {
            todo!("Single artist option not yet implemented")
        }
    }

    section.tracks = plex
        .fetch_music(filters, section.get_sorting(), Some(1111))
        .await?;

    section.run_manual_filters(time_limit);

    Ok(())
}

#[derive(Clone, Debug, Default, DefaultBuilder, Deserialize, PartialEq, Serialize)]
pub struct ProfileSection {
    /// Deduplicate tracks by its `guid`, so that the exact same track that appears on
    /// multiple albums (e.g., a studio album and a Greatest Hits album) only appears once in
    /// the resulting playlist.
    deduplicate_tracks_by_guid: bool,
    deduplicate_tracks_by_title_and_artist: bool,
    enabled: bool,
    /// Caps the number of tracks by an artist that can appear in a single playlist.
    /// A value of `0` allows for an unlimited number of tracks.
    maximum_tracks_by_artist: i32,
    minimum_track_rating: u32,
    randomize_tracks: bool,
    section_type: SectionType,
    sorting: String,
    #[serde(skip)]
    tracks: Vec<Track>,
}

impl ProfileSection {
    pub fn with_enabled() -> Self {
        Self::default().enabled(true)
    }

    pub fn get_deduplicate_tracks_by_guid(&self) -> bool {
        self.deduplicate_tracks_by_guid
    }

    pub fn has_maximum_tracks_by_artist(&self) -> bool {
        self.maximum_tracks_by_artist > 0
    }

    pub fn get_maximum_tracks_by_artist(&self) -> i32 {
        self.maximum_tracks_by_artist
    }

    pub fn get_minimum_track_rating(&self) -> u32 {
        (self.minimum_track_rating - 1) * 2
    }

    pub fn get_sorting(&self) -> Vec<&str> {
        self.sorting.split(',').collect::<_>()
    }

    pub fn get_section_type(&self) -> SectionType {
        self.section_type
    }

    fn is_section_type(&self, section_type: SectionType) -> bool {
        self.section_type == section_type
    }

    pub fn is_unplayed(&self) -> bool {
        self.is_section_type(SectionType::Unplayed)
    }

    pub fn is_least_played(&self) -> bool {
        self.is_section_type(SectionType::LeastPlayed)
    }

    pub fn is_oldest(&self) -> bool {
        self.is_section_type(SectionType::Oldest)
    }

    pub fn set_tracks(&mut self, tracks: Vec<Track>) {
        self.tracks = tracks
    }

    pub fn num_tracks(&self) -> usize {
        self.tracks.len()
    }

    pub fn run_manual_filters(&mut self, time_limit: f64) {
        self.deduplicate_by_track_guid();
        self.run_deduplicate_by_title_and_artist();
        self.limit_tracks_by_artist();
        self.sort_tracks();
        self.reduce_to_time_limit(time_limit);

        if self.randomize_tracks {
            let mut rng = ChaCha8::new();
            rng.shuffle(&mut self.tracks);
        }
    }

    fn deduplicate_by_track_guid(&mut self) {
        if self.deduplicate_tracks_by_guid {
            self.tracks.dedup_by_key(|track| track.guid.to_owned());
        }
    }

    fn run_deduplicate_by_title_and_artist(&mut self) {
        if self.deduplicate_tracks_by_title_and_artist {
            self.tracks
                .par_sort_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
            self.tracks
                .dedup_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
        }
    }

    fn limit_tracks_by_artist(&mut self) {
        if self.maximum_tracks_by_artist == 0 {
            return;
        }

        if self.is_unplayed() || self.is_least_played() {
            self.tracks
                .par_sort_by_key(|track| (track.view_count, track.last_played()))
        } else {
            self.tracks
                .par_sort_by_key(|track| (track.last_played(), track.view_count))
        }

        let mut artist_occurrences: HashMap<&str, i32> = HashMap::new();
        for track in self.tracks.clone().iter() {
            let artist_guid = track.artist_guid();
            let occurrences = artist_occurrences.entry(artist_guid).or_insert(0);
            *occurrences += 1;

            if *occurrences >= self.maximum_tracks_by_artist {
                let index = self
                    .tracks
                    .par_iter()
                    .position_first(|t| t == track)
                    .expect("Index not found");
                self.tracks.remove(index);
            }
        }
    }

    fn sort_tracks(&mut self) {
        if self.is_least_played() {
            self.tracks
                .par_sort_by_key(|t| (t.plays(), t.last_played()))
        } else if self.is_oldest() {
            self.tracks
                .par_sort_by_key(|t| (t.last_played(), t.plays()))
        }
    }

    fn dedup_tracks_by_list(&mut self, comp: &[Track]) {
        dedup_lists(&mut self.tracks, comp)
    }

    pub fn reduce_to_time_limit(&mut self, time_limit: f64) {
        self.tracks = get_tracks_within_time_range(&self.tracks, time_limit)
    }
}

fn dedup_lists(lst: &mut Vec<Track>, comp: &[Track]) {
    for a in lst.clone().iter() {
        for b in comp {
            if a.id() == b.id() {
                let index = lst.par_iter().position_first(|t| t == a).unwrap();
                lst.remove(index);
            }
        }
    }
}

fn get_tracks_within_time_range(tracks: &[Track], time_limit: f64) -> Vec<Track> {
    let limit = TimeDelta::seconds((time_limit * 60_f64 * 60_f64) as i64);

    let mut total = TimeDelta::seconds(0);
    let index = tracks
        .iter()
        .position(|track| {
            total += TimeDelta::milliseconds(track.duration());
            total > limit
        })
        .unwrap_or(0);

    tracks[..=index].to_vec()
}

// TESTS ######################################################################
