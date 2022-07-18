export RUST_LOG=debug
cargo run --bin hexstody-hot -- \
	--operator-public-keys operator1-key.pub.pem operator2-key.pub.pem \
	--start-regtest serve
