# Partikel Makefile

.PHONY: all build check test clippy fmt install clean mcp eval

all: check test

build:
	cargo build --release

check:
	cargo check --all-targets

test:
	cargo test --all-targets

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

install:
	cargo install --path .

eval:
	cargo run --bin ptk -- eval

mcp:
	cargo run --bin ptk -- mcp

clean:
	cargo clean

coverage:
	cargo llvm-cov --html --output-dir ./coverage_report
	@echo "Coverage HTML report generated at ./coverage_report/html/index.html"

coverage-open:
	cargo llvm-cov --html --open
