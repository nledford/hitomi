use std::collections::HashMap;

use chrono::TimeDelta;
use log::info;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator};
use rayon::prelude::ParallelSliceMut;

use crate::playlists::combined_playlist::CombinedPlaylist;
use crate::playlists::essential_artists::EssentialArtistsPlaylist;
use crate::playlists::random_playlist::RandomPlaylist;
use crate::plex::models::Track;
use crate::state::APP_STATE;

mod combined_playlist;
mod essential_artists;
mod random_playlist;

pub async fn build_playlists(looping: bool) -> anyhow::Result<()> {
    // Refresh standard playlists
    CombinedPlaylist::build().await?;
    EssentialArtistsPlaylist::build().await?;

    // random playlist only refreshes every half-hour
    if APP_STATE.lock().refresh_random_playlist() || !APP_STATE.lock().first_run() {
        RandomPlaylist::build().await?;
    }

    // If looping, display status info
    if looping {
        print_next_refresh();
    }

    // Playlists have been refreshed at least once
    {
        let mut state = APP_STATE.lock();
        state.set_first_run();
    }

    Ok(())
}

/*
fn remove_duplicates(
    tracks: &mut Vec<Track>,
    remove_duplicate_tracks: bool,
    limit_tracks_by_artist: bool,
    is_combined: bool,
    least_tracks: bool,
) {
    let before = tracks.len();

    // De-duplicate by track guid
    tracks.dedup_by_key(|track| track.guid.to_owned());

    if remove_duplicate_tracks {
        tracks.par_sort_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
        tracks.dedup_by_key(|track| (track.title().to_owned(), track.artist().to_owned()));
    }

    if limit_tracks_by_artist {
        if least_tracks {
            tracks.par_sort_by_key(|track| (track.view_count, track.last_played()))
        } else {
            tracks.par_sort_by_key(|track| (track.last_played(), track.view_count))
        }

        let mut artist_occurrences: HashMap<&str, i32> = HashMap::new();
        for track in tracks.clone().iter() {
            let artist_guid = track.artist_guid();
            let occurrences = artist_occurrences.entry(artist_guid).or_insert(0);
            *occurrences += 1;

            if *occurrences >= 25 {
                let index = tracks
                    .par_iter()
                    .position_first(|t| t == track)
                    .expect("Index not found");
                tracks.remove(index);
            }
        }
    }

    let after = tracks.len();

    if !is_combined {
        info!("\tBEFORE: {} tracks | AFTER: {} tracks", before, after)
    }
}
*/

// fn dedup_lists(lst: &mut Vec<Track>, comp: &[Track]) {
//     for a in lst.clone().iter() {
//         for b in comp {
//             if a.id() == b.id() {
//                 let index = lst.par_iter().position_first(|t| t == a).unwrap();
//                 lst.remove(index);
//             }
//         }
//     }
// }

fn print_next_refresh() {
    let mut lock = APP_STATE.lock();
    let next_refresh = lock.get_next_refresh_time().format("%H:%M");

    info!("Next refresh at {}\n", next_refresh);
}
