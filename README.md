# How to build
You need to run local PostgreSQL instance to allow compiler to check SQL quieries in advance:
1. Create `hexstody` user with `hexstody` password and allow to create databases for tests:
```
create role hexstody login createdb password 'hexstody';
create database hexstody owner hexstody;
```
2. Run `./hexstody-db/migrate.sh`;
3. Set `DATABASE_URL` env:
```
export DATABASE_URL=postgresql://hexstody:hexstody@localhost:5432/hexstody
```

How to build and run:

Tests
```
cargo test
```

Hot wallet
```
cargo run --bin hexstody-hot -- serve
```
Hot wallet with builtin regtest BTC node:
```
cargo run --bin hexstody-hot -- --start-regtest serve
```

Operator key generation tool
```
cargo run --bin operator-keygen
```
