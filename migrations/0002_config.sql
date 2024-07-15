drop table if exists config;
create table if not exists config
(
    name  text not null,
    value text
);