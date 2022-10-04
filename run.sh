export RUST_LOG=debug
cargo run --bin hexstody-btc serve \
	--operator-public-keys operator1-key.pub.pem operator2-key.pub.pem \
	--node-password bitcoin \
	--hot-domain localhost \
	--cold-address bc1q0aew83ah388l9jgm94c7pw3rf8mem8d7llxdsr \
	--cold-sat 10000000 \
