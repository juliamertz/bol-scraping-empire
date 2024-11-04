build:
	cargo build --release
	-mkdir dist
	cp target/release/bol-scraper-empire dist
	cp scripts/run.sh dist
