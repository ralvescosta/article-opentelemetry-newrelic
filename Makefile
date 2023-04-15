http-server:
	@RUST_ENV=local APP_NAME=http-server cargo run --bin http-server

consumers:
	@RUST_ENV=local APP_NAME=http-server cargo run --bin consumers