use bevy::prelude::*;
use game_lib::GamePlugin;

#[bevy_main]
fn main() {
	App::new()
		.add_plugins(GamePlugin {
			asset_dir: "assets",
			imported_asset_dir: "imported_assets/Default",
		})
		.run()
}
