create table if not exists config
(
    id_          integer primary key autoincrement,
    namespace_id varchar(100) not null,
    id           varchar(500) not null,
    content      text         not null,
    ts           timestamp    not null default current_timestamp,
    description  varchar(500),
    unique (namespace_id, id)
);
create table if not exists config_history
(
    id_          integer primary key autoincrement,
    namespace_id varchar(100) not null,
    id           varchar(500) not null,
    content      text         not null,
    ts           timestamp    not null,
    description  varchar(500)
);
