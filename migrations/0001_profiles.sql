-- Profiles table

drop table if exists profile;
CREATE TABLE profile
(
    profile_id        integer                       not null
        constraint profile_pk
            primary key autoincrement,
    playlist_id TEXT not null,
    profile_title     TEXT    default 'New Profile' not null,
    profile_summary   TEXT,
    enabled           boolean default 1             not null,
    profile_source    TEXT                          not null,
    profile_source_id TEXT,
    refresh_interval  integer default 5             not null,
    time_limit        integer default 0,
    track_limit       integer default 0,
    constraint enabled_boolean
        check (profile.enabled in (0, 1)),
    constraint profile_source
        check (profile.profile_source in ('Library', 'Collection', 'Playlist', 'SingleArtist')),
    constraint refresh_interval
        check (profile.refresh_interval in (2, 3, 4, 5, 6, 10, 12, 15, 20, 30)),
    constraint time_limit
        check (profile.time_limit >= 0),
    constraint track_limit
        check (profile.track_limit >= 0)
);

CREATE UNIQUE INDEX profile_profile_title_uindex
    on profile (profile_title);

-- Profile sections table

drop table if exists profile_section;
CREATE TABLE profile_section
(
    profile_section_id                     integer           not null
        constraint profile_section_pk
            primary key autoincrement,
    profile_id                             integer           not null
        constraint profile_section_profile_profile_id_fk
            references profile on delete cascade,
    section_type                           text              not null,
    enabled                                boolean default 1 not null,
    deduplicate_tracks_by_guid             boolean default 0 not null,
    deduplicate_tracks_by_title_and_artist boolean default 0 not null,
    maximum_tracks_by_artist               integer default 0 not null,
    minimum_track_rating                   integer default 0 not null,
    randomize_tracks                       boolean default 0 not null,
    sorting                                TEXT              not null,
    constraint deduplicate_tracks_by_guid
        check (profile_section.deduplicate_tracks_by_guid in (0, 1)),
    constraint deduplicate_tracks_by_title_and_artist
        check (profile_section.deduplicate_tracks_by_title_and_artist in (0, 1)),
    constraint enabled
        check (profile_section.enabled in (0, 1)),
    constraint maximum_tracks_by_artist
        check (profile_section.maximum_tracks_by_artist >= 0),
    constraint minimum_track_rating
        check (profile_section.minimum_track_rating >= 0 AND profile_section.minimum_track_rating <= 5),
    constraint randomize_tracks
        check (profile_section.randomize_tracks in (0, 1)),
    constraint section_type
        check (profile_section.section_type in ('Unplayed', 'LeastPlayed', 'Oldest'))
);

-- Profiles view

drop view if exists v_profile;
create view v_profile as
select profile_id,
       playlist_id,
       profile_title,
       profile_summary,
       enabled,
       profile_source,
       profile_source_id,
       refresh_interval,
       time_limit,
       track_limit,
       num_sections,
       num_sections >= 3 has_max_sections,
       (cast(time_limit as real) / cast(num_sections as real))                   section_time_limit,
       cast((60.0 / refresh_interval) as integer)                                refreshes_per_hour,
       datetime(strftime('%s', current_timestamp) - (strftime('%s', current_timestamp) % (refresh_interval * 60.0)),
                'unixepoch',
                'localtime')                                                     current_refresh,
       datetime(strftime('%s', current_timestamp) +
                ((refresh_interval * 60.0) - (strftime('%s', current_timestamp)) % (refresh_interval * 60.0)),
                'unixepoch', 'localtime')                                        next_refresh_at,
       (cast(strftime('%M', current_timestamp) as real) % refresh_interval == 0) eligible_for_refresh
from (select profile_id,
             playlist_id,
             profile_title,
             profile_summary,
             enabled,
             profile_source,
             profile_source_id,
             refresh_interval,
             case when time_limit == 0 then 365 * 24 else time_limit end time_limit,
             track_limit,
             (select count(1)
              from profile_section ps
              where profile_id = p.profile_id
                and p.enabled = 1
                and ps.enabled = 1)                                      num_sections
      from profile p
      order by profile_title);

