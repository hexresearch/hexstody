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

Operator key generation tool
```
cargo run --bin operator-keygen -- -o operator-key-1 -p
cargo run --bin operator-keygen -- -o operator-key-2 -p
```

## Bitcoin regtest network
When `--start-regtest` flag is specified, `hexstody-hot` automatically starts 2 connected BTC nodes and the `hexstody-btc` API instance.
By default BTC nodes use 9804 and 9806 RPC ports. The default RPC password and user is "regtest". The first node is used by the hexstody itself, while you can use the second node for depositing and withdrawing funds.
```
cargo run --bin hexstody-hot -- --operator-public-keys operator-key-1.pub.pem operator-key-2.pub.pem --start-regtest serve
```

Then you can interact with BTC nodes via `bitcoin-cli`.

Here are some usefull commands
```
bitcoin-cli -rpcuser=regtest -rpcpassword=regtest -rpcport=9806 -generate 101
bitcoin-cli -rpcuser=regtest -rpcpassword=regtest -rpcport=9806 -named sendtoaddress address="bcrt1q32ykh6yllg055v0ev7e39sguvqhft39j4tedg8" amount=1
bitcoin-cli -rpcuser=regtest -rpcpassword=regtest -rpcport=9806 getnewaddress
```

## ETH testnet node adapter

run shell:
```
nix-shell
```
after entering into nix-shell run to start eth adatper
```
./runeth.sh
```

# Tips and tricks

Run vscode with export to allow rust extension to validate sqlx macros

```
(export DATABASE_URL=postgres://hexstody:hexstody@localhost/hexstody; code)
```

Run sass watcher to auto-compile hexstody-public/static/css/styles.scss on every change:

```
sass --watch --sourcemap=none hexstody-public/static/css/styles.scss:hexstody-public/static/css/styles.css
```

Omit `--watch` to compile once. `sass` is available in nix-shell 
