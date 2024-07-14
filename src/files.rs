use std::path::Path;

use anyhow::Result;
use simplelog::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::profiles::profile::Profile;

pub async fn save_profile_to_disk(profile: &Profile, profiles_directory: &str) -> Result<()> {
    tokio::fs::create_dir_all(profiles_directory).await?;

    let json = serde_json::to_string_pretty(profile)?;
    let mut file = tokio::fs::File::create(profile.get_profile_path(profiles_directory).await).await?;
    file.write_all(json.as_bytes()).await?;

    Ok(())
}

async fn load_profile_from_disk(path: &str) -> Result<Profile> {
    match tokio::fs::File::open(path).await {
        Ok(file) => {
            let mut file = file;
            let mut profile = String::default();
            file.read_to_string(&mut profile).await?;
            let profile: Profile = serde_json::from_str(&profile)?;
            Ok(profile)
        }
        Err(err) => {
            panic!("Error attempting to load profile from disk: {err}")
        }
    }
}

pub async fn load_profiles_from_disk(dir: &str) -> Result<Vec<Profile>> {
    debug!("Loading profiles from disk...");
    let dir = Path::new(dir);

    if !dir.exists() {
        panic!("Profiles directory `{}` could not be found.", dir.display())
    }

    if !dir.is_dir() {
        panic!("Profiles directory `{}` is not a directory.", dir.display())
    }

    if dir.read_dir()?.next().is_none() {
        error!("Profiles directory `{}` is empty.", dir.display());
        return Ok(vec![]);
    }

    let mut result = vec![];

    let mut reader = tokio::fs::read_dir(&dir).await?;
    while let Some(entry) = reader.next_entry().await? {
        let profile = load_profile_from_disk(entry.path().to_str().unwrap()).await?;
        result.push(profile)
    }

    info!("{} profile(s) loaded from disk", &result.len());

    Ok(result)
}
