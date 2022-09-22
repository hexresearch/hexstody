create table users_eth(
    login text not null primary key,
    address text not null,
    data jsonb
);
