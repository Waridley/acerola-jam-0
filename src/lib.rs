use bevy::prelude::*;

pub struct GamePlugin {
	pub asset_dir: &'static str,
	pub imported_asset_dir: &'static str,
}

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(DefaultPlugins.set(AssetPlugin {
			file_path: self.asset_dir.to_owned(),
			processed_file_path: self.imported_asset_dir.to_owned(),
			mode: AssetMode::Processed,
			..default()
		})).add_systems(Startup, hello_world);
	}
}

pub fn hello_world() {
	info!("Hello, Acerola Game Jam #0!");
}