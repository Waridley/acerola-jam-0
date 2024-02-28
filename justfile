dev:
	cargo run --package="sonday-acerola-jam-0-desktop" --profile="desktop" --features="debugging,bevy/file_watcher,bevy/asset_processor,bevy_dylib"

release:
	cargo run --package="sonday-acerola-jam-0-desktop" --profile="desktop-release"

release-debug:
	cargo run --package="sonday-acerola-jam-0-desktop" --profile="release-debug" --features="debugging"

web:
	trunk serve --public-url="/" --open --features="debugging"

web-release:
	trunk serve --public-url="/" --open --release

web-release-debug:
	trunk serve --public-url="/" --open --profile="release-debug" --features="debugging"

test:
	cargo test --workspace --features="vis_test"

headless-test:
	cargo test --workspace --features="testing"
