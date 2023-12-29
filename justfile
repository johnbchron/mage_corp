
# run the project
run:
	MTL_HUD_ENABLED=1 cargo run --package mage_corp

run-release:
	MTL_HUD_ENABLED=1 cargo run --release --package mage_corp

test:
	cargo nextest run
	
# run the pre-commit checklist
pre-c:
	cargo fmt
	cargo clippy
	cargo test	
	
