create table updates(
    id serial primary key,
    created timestamp not null,
    version smallint not null,
    tag text not null,
    body jsonb not null
);

create table users_eth(
    login text not null primary key,
    address text not null,
    data jsonb
);
