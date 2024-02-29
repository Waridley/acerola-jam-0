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
		}).set(WindowPlugin {
			primary_window: Some(Window {
				title: "Sonday Studios -- Acerola Jam #0".to_owned(),
				resizable: true,
				canvas: Some("#game_canvas".to_owned()),
				..default()
			}),
			..default()
		})).add_systems(Startup, hello_world);
	}
}

pub fn hello_world() {
	info!("Hello, Acerola Game Jam #0!");
}