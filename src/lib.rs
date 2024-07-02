/*!
`hitomi` is a CLI application that generates custom playlists on [Plex](https://plex.tv) servers using `.json` profiles.
 */

pub mod cli;
pub mod config;
pub mod http_client;
pub mod plex;
pub mod profiles;
pub mod state;
pub mod types;
pub mod utils;
