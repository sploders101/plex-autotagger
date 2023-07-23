build:
	cargo build

test:
	cargo test

test_examples:
	cargo run --example basic
	cargo run --example find_by_imdb_id
	cargo run --example append_to_response

lint:
	cargo clippy --all-targets --all-features -- -D warnings

check_style:
	cargo fmt -- --check

publish:
	cargo login $(CRATES_TOKEN)
	cargo package
	cargo publish

setup:
	rustup toolchain install stable
	rustup default stable
	rustup update
	rustup component add clippy rustfmt

clean:
	cargo clean