/*!
`hitomi` is a CLI application that generates custom playlists on [Plex](https://plex.tv) servers using `.json` profiles.
 */

pub mod cli;
pub mod config;
pub mod db;
pub mod http_client;
pub mod logger;
pub mod plex;
pub mod profiles;
pub mod types;
pub mod utils;
