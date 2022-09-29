export RUST_LOG=debug
cargo run --bin hexstody-hot -- \
	--operator-public-keys operator1-key.pub.pem operator2-key.pub.pem \
	--public-api-domain https://demo.desolator.net \
	--operator-api-domain https://operator.desolator.net \
	--start-regtest serve
