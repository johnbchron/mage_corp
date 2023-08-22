
# run the project
run:
	cargo run
	
# run the pre-commit checklist
pre-c:
	cargo fmt
	cargo clippy
	cargo test	
	
