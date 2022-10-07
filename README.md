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
By default BTC nodes use 9804 and 9806 RPC ports. The default RPC password and user is "regtest".
The first node is used by the hexstody itself, while you can use the second node for depositing and withdrawing funds.
*NOTE*: tis is important to put some flag after the `--operator-public-keys` flag and between the `serve` command so parser knows that the operator keys list is over.
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

## Full testnet setup
In this section we will describe the whole process of starting `hexstody` on testnet network.
The main part of the application is `hexstody-hot` package. 
To run it, you first need to run `hexstody-btc` and `hexstody-eth` packages.
`hexstody-btc` and `hexstody-eth`, in turn, require Bitcoin node and Ethereum node to be running.
In total we need to start 5 different services:
- Bitcoin node
- Ethereum node
- `hexstody-btc`
- `hexstody-eth`
- `hexstody-hot`
Each of them we will start in a separate terminal.

Let's start by launching the Bitcoin node:
```
bitcoind -testnet -server -rpcuser=testnet -rpcpassword=testnet -rpcport=8332
```
 
Now let's create a Bitcoin wallet and generate cold wallet address:
```
bitcoin-cli -rpcuser=testnet -rpcpassword=testnet -rpcport=8332 createwallet testwallet
bitcoin-cli -rpcuser=testnet -rpcpassword=testnet -rpcport=8332 getnewaddress
```
Copy the result and use it as a cold wallet address in the next step

Then start `hexstody-btc` service:
```
cargo run --bin hexstody-btc -- serve \
    --network testnet \
    --node-url http://127.0.0.1:8332/wallet/testwallet
    --node-user testnet --node-password testnet \
    --hot-domain 127.0.0.1:8180 \
    --operator-public-keys operator-key-1.pub.pem operator-key-2.pub.pem \
    --cold-sat 100000000 --cold-address tb1qmv63peuryphwkhs3jc6mfvx8ncs99x647fsece
```

To start Ethereum node clone `hexstody/eth-sandbox` repo and type:
```
nix-shell
./run.sh
```

Start `hexstody-eth` service:
```
./runeth.sh
```

And finally start `hexstody-hot`:
```
cargo run --bin hexstody-hot -- \
    --operator-public-keys operator-key-1.pub.pem operator-key-2.pub.pem \
    --network testnet serve
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
