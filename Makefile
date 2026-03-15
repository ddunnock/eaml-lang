.PHONY: build test check fmt clean

build:
	cargo build --workspace

test:
	cargo test --workspace
	cd python && uv run pytest || test $$? -eq 5

check:
	cargo check --workspace
	cargo clippy --workspace -- -D warnings
	cd python && uv run ruff check .
	cd python && uv run mypy src/

fmt:
	cargo fmt --all
	cd python && uv run ruff format .

# Run the compiler on a specific file
run FILE:
	cargo run -p eaml-cli -- compile $(FILE)

# Snapshot test review
review:
	cargo insta review

clean:
	cargo clean
	find . -name __pycache__ -exec rm -rf {} +