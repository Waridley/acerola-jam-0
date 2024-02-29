use bevy::prelude::*;
use data::DataPlugin;
use time_graph::TimeGraphPlugin;

pub mod data;
pub mod time_graph;

pub struct GamePlugin {
	pub asset_dir: &'static str,
	pub imported_asset_dir: &'static str,
}

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			DefaultPlugins
				.set(AssetPlugin {
					file_path: self.asset_dir.to_owned(),
					processed_file_path: self.imported_asset_dir.to_owned(),
					mode: AssetMode::Processed,
					..default()
				})
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: "Sonday Studios -- Acerola Jam #0".to_owned(),
						resizable: true,
						canvas: Some("#game_canvas".to_owned()),
						..default()
					}),
					..default()
				}),
			DataPlugin,
			TimeGraphPlugin,
		))
		.add_systems(Startup, setup);
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct GlobalsScene(pub Handle<DynamicScene>);

pub fn setup(
	mut cmds: Commands,
	assets: Res<AssetServer>,
	mut scene_spawner: ResMut<SceneSpawner>,
) {
	let globals_scene = assets.load("globals.scn.ron");
	cmds.insert_resource(GlobalsScene(globals_scene.clone()));
	scene_spawner.spawn_dynamic(globals_scene);
}
