/*!
`hitomi` is an application that generates custom playlists on [Plex](https://plex.tv) servers using `.json` profiles.
 */

pub mod app;
pub mod config;
pub mod db;
pub mod event;
pub mod handler;
pub mod http_client;
pub mod logger;
pub mod plex;
pub mod profiles;
pub mod tui;
pub mod types;
pub mod ui;
pub mod utils;
