use bevy::utils::{SystemTime, Duration, HashMap};
use bevy::prelude::*;
use crate::data::tl::{Moment, Timeline, Timelines};

pub mod data;

pub struct GamePlugin {
	pub asset_dir: &'static str,
	pub imported_asset_dir: &'static str,
}

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins(
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
			)
			.add_systems(Startup, setup)
			.add_systems(Update, print_timelines.run_if(resource_exists_and_changed::<Timelines>))
			.register_type::<Moment>()
			.register_type::<HashMap<Duration, Moment>>()
			.register_type::<Timeline>()
			.register_type::<Vec<Timeline>>()
			.register_type::<Timelines>();
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct GlobalsScene(pub Handle<DynamicScene>);

pub fn setup(mut cmds: Commands, assets: Res<AssetServer>, mut scene_spawner: ResMut<SceneSpawner>) {
	let globals_scene = assets.load("globals.scn.ron");
	cmds.insert_resource(GlobalsScene(globals_scene.clone()));
	scene_spawner.spawn_dynamic(globals_scene);
}

pub fn print_timelines(timelines: Res<Timelines>) {
	info!("{timelines:?}");
}
