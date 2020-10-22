CC=musl-CC
cargo build --release --target armv7-unknown-linux-musleabihf
cp ./target/armv7-unknown-linux-musleabihf/release/backend ./build/backend