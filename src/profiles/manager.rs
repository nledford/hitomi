use std::cmp::PartialEq;

use itertools::Itertools;
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use uuid::Uuid;

use crate::profiles::profile::Profile;
use crate::profiles::profile_section::ProfileSection;

new_key_type! {
    pub struct ProfileKey;
}

#[derive(Clone, Debug)]
pub struct ProfileManager {
    /// Profiles that have been loaded from disk
    profiles: Vec<Profile>,
    /// Profiles being managed by the application
    managed_profiles: SlotMap<ProfileKey, Profile>,
    managed_unplayed_sections: SecondaryMap<ProfileKey, ProfileSection>,
    managed_least_played_sections: SecondaryMap<ProfileKey, ProfileSection>,
    managed_oldest_sections: SecondaryMap<ProfileKey, ProfileSection>,
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self {
            profiles: vec![],
            managed_profiles: SlotMap::default(),
            managed_unplayed_sections: SecondaryMap::default(),
            managed_least_played_sections: SecondaryMap::default(),
            managed_oldest_sections: SecondaryMap::default(),
        }
    }
}

impl ProfileManager {
    pub fn new(profiles: Vec<Profile>) -> Self {
        let mut manager = ProfileManager::default();
        manager.profiles = profiles;
        manager.build_managed_profiles_and_sections();
        manager
    }

    fn build_managed_profiles_and_sections(&mut self) {
        let mut managed_profiles = SlotMap::with_key();
        let mut managed_unplayed_sections = SecondaryMap::new();
        let mut managed_least_played_sections = SecondaryMap::new();
        let mut managed_oldest_sections = SecondaryMap::new();

        for profile in &self.profiles {
            let profile_key = managed_profiles.insert(profile.clone());

            for section in profile.get_sections() {
                if section.is_unplayed_section() {
                    let _ = managed_unplayed_sections.insert(profile_key, section.clone());
                }

                if section.is_least_played_section() {
                    let _ = managed_least_played_sections.insert(profile_key, section.clone());
                }

                if section.is_oldest_section() {
                    let _ = managed_oldest_sections.insert(profile_key, section.clone());
                }
            }
        }

        self.managed_profiles = managed_profiles;
        self.managed_unplayed_sections = managed_unplayed_sections;
        self.managed_least_played_sections = managed_least_played_sections;
        self.managed_oldest_sections = managed_oldest_sections;
    }

    fn get_num_profiles(&self) -> usize {
        self.managed_profiles.len()
    }

    pub fn have_profiles(&self) -> bool {
        !self.managed_profiles.is_empty()
    }

    pub fn get_profiles(&self) -> Vec<&Profile> {
        if !self.have_profiles() {
            return vec![];
        }

        self.managed_profiles
            .iter()
            .map(|(k, v)| v)
            .sorted_unstable_by_key(|profile| profile.get_title().to_owned())
            .collect::<Vec<_>>()
    }

    pub fn get_enabled_profiles(&self) -> Vec<&Profile> {
        self.get_profiles()
            .into_iter()
            .filter_map(|profile| {
                if profile.get_enabled() {
                    Some(profile)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn get_profiles_to_refresh(&self, ran_once: bool) -> Vec<&Profile> {
        if ran_once && !self.get_any_profile_refresh() {
            return vec![];
        }

        // If the application has run once, we DO NOT want to override refreshing profiles
        self.get_enabled_profiles()
            .into_iter()
            .filter(|profile| profile.check_for_refresh(!ran_once))
            .collect::<Vec<_>>()
    }

    pub fn get_profile_titles(&self) -> Vec<String> {
        self.get_profiles()
            .iter()
            .map(|profile| profile.get_title().to_string())
            .collect::<Vec<_>>()
    }

    pub fn get_profile_by_id(&self, id: Uuid) -> Option<&Profile> {
        self.get_profiles()
            .into_iter()
            .find(|profile| profile.get_profile_id() == id)
    }

    pub fn get_profile_by_title(&self, title: &str) -> Option<&Profile> {
        self.get_profiles()
            .into_iter()
            .find(|profile| profile.get_title() == title)
    }

    pub fn list_profiles(&self) {
        let titles = self.get_profile_titles();
        if titles.is_empty() {
            println!("No profiles found.")
        } else {
            println!("Existing profiles found");
            for title in titles {
                println!("  - {}", title)
            }
        }
    }

    pub fn list_profiles_and_sections(&self) {
        let profiles = &self.managed_profiles;

        for (k, v) in profiles {
            println!("{}", v.get_title());

            if self.managed_unplayed_sections.contains_key(k) {
                println!(" - Unplayed")
            }

            if self.managed_least_played_sections.contains_key(k) {
                println!(" - Least Played")
            }

            if self.managed_oldest_sections.contains_key(k) {
                println!(" - Oldest")
            }
        }

        // let sections = &self.managed_profile_sections;
        // for (k, v) in profiles {
        //     println!("{}", v.get_title());
        //
        //     for (_, section) in sections.iter().filter(|(sk, sv)| *sk == k) {
        //         println!(" - {}", section.get_section_type())
        //     }
        // }
    }

    fn get_any_profile_refresh(&self) -> bool {
        self.get_enabled_profiles()
            .iter()
            .any(|profile| profile.check_for_refresh(false))
    }
}