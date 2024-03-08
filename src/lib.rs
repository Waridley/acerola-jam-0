use crate::{
	cam::CamPlugin,
	data::SystemRegistry,
	happens::HappeningsPlugin,
	player::PlayerPlugin,
	scn::{
		clock::{tick_hand, Hand},
		EnvironmentPlugin,
	},
	ui::GameUiPlugin,
};
use bevy::{prelude::*, reflect::TypeRegistryArc};
use bevy_xpbd_3d::{plugins::PhysicsPlugins, prelude::Gravity};
use data::DataPlugin;
use std::sync::OnceLock;
use time_graph::TimeGraphPlugin;

pub mod cam;
pub mod data;
pub mod happens;
pub mod player;
pub mod scn;
pub mod time_graph;
pub mod ui;

pub static TYPE_REGISTRY: OnceLock<TypeRegistryArc> = OnceLock::new();
pub fn type_registry() -> &'static TypeRegistryArc {
	TYPE_REGISTRY
		.get()
		.expect("TYPE_REGISTRY should be initialized")
}

pub static ASSET_SERVER: OnceLock<AssetServer> = OnceLock::new();
pub fn asset_server() -> &'static AssetServer {
	ASSET_SERVER
		.get()
		.expect("ASSET_SERVER should be initialized")
}

pub struct GamePlugin {
	pub asset_dir: &'static str,
	pub imported_asset_dir: &'static str,
}

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		// Dependencies
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
			PhysicsPlugins::default(),
		));

		TYPE_REGISTRY
			.set(app.world.resource::<AppTypeRegistry>().0.clone())
			.expect("AppTypeRegistry already set");
		ASSET_SERVER
			.set(app.world.resource::<AssetServer>().clone())
			.expect("AssetServer already set");
		app.init_resource::<SystemRegistry>();
		app.insert_resource(Gravity(Vec3::NEG_Z * 9.81));

		// Mine
		app.add_plugins((
			DataPlugin,
			CamPlugin,
			HappeningsPlugin,
			TimeGraphPlugin,
			PlayerPlugin,
			EnvironmentPlugin,
			GameUiPlugin,
		))
		.add_systems(Startup, setup)
		.add_systems(Update, tick_hand)
		.register_type::<Hand>();

		#[cfg(feature = "debugging")]
		app.add_plugins(bevy_xpbd_3d::plugins::PhysicsDebugPlugin::default())
			.insert_gizmo_group(
				bevy_xpbd_3d::prelude::PhysicsGizmos::default(),
				GizmoConfig {
					enabled: false,
					..default()
				},
			)
			.add_systems(Update, toggle_phys_gizmos);
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

#[cfg(feature = "debugging")]
pub fn toggle_phys_gizmos(mut store: ResMut<GizmoConfigStore>, keys: Res<ButtonInput<KeyCode>>) {
	if keys.just_pressed(KeyCode::KeyG) {
		let (gizmos, _phys) = store.config_mut::<bevy_xpbd_3d::prelude::PhysicsGizmos>();
		gizmos.enabled = !gizmos.enabled;
	}
}
