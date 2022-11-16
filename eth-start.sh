export RUST_LOG=debug
cargo run --bin hexstody-eth -- serve \
    --network bitcoin \
    --node-url http://127.0.0.1:8332/wallet/default \
    --node-user user --node-password bitcoin \
    --hot-domain 127.0.0.1:8180 \
    --operator-public-keys operator1-key.pub.pem operator2-key.pub.pem \
    --cold-sat 100000000 --cold-address bc1qq3xwggzdg8yr0c3v2s59kgwe980eu6jtt8qksg
