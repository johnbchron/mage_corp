
# run the project
run:
	cargo run --package mage_corp

run-release:
	cargo run --release --package mage_corp

test:
	cargo nextest run
	
# run the pre-commit checklist
pre-c:
	cargo fmt
	cargo clippy
	cargo test	
	
