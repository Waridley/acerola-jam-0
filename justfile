dev:
	cargo run --package="sonday-acerola-jam-0-desktop" --profile="desktop" --features="debugging,bevy/file_watcher,bevy/asset_processor,bevy_dylib"

release:
	cargo run --package="sonday-acerola-jam-0-desktop" --profile="desktop-release"

release-debug:
	cargo run --package="sonday-acerola-jam-0-desktop" --profile="release-debug" --features="debugging"

serve:
	trunk serve --open --public-url="/" --features="debugging"

web:
	trunk build --public-url="/" --features="debugging"

web-release:
	trunk build --public-url="/" --release

web-release-debug:
	trunk build --public-url="/" --profile="release-debug" --features="debugging"

test:
	cargo test --workspace --features="vis_test"

headless-test:
	cargo test --workspace --features="testing"

fmt:
	cargo fmt --all -- --config imports_granularity="Crate"
