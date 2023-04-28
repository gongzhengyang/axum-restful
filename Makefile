.PHONY: fmt
fmt:
	cargo fmt
	cargo clippy --fix --allow-dirty --all-targets
