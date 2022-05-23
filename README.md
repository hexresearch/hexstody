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

Public API
```
export ROCKET_TEMPLATE_DIR=hexstody-hot-public/templates
export ROCKET_SECRET_KEY="LOe6Tf3P5DU6u7TgiCy9dSzd/b/6qyPL0wdDPfy56Wo="
cargo run --bin hexstody-hot-public -- serve
```

Operator key generation tool
```
cargo run --bin operator-keygen -- --password
```

Operator API
```
export ROCKET_TEMPLATE_DIR=hexstody-hot-operator/templates
export ROCKET_SECRET_KEY="LOe6Tf3P5DU6u7TgiCy9dSzd/b/6qyPL0wdDPfy56Wo="
cargo run --bin hexstody-hot-operator -- serve
```