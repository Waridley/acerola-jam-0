use crate::{cam::CamPlugin, happens::HappeningsPlugin};
use bevy::prelude::*;
use bevy_xpbd_3d::{
	plugins::PhysicsPlugins,
	prelude::{Collider, RigidBody},
};
use data::DataPlugin;
use time_graph::TimeGraphPlugin;

pub mod cam;
pub mod data;
pub mod happens;
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
			CamPlugin,
			HappeningsPlugin,
			TimeGraphPlugin,
			PhysicsPlugins::default(),
		))
		.add_systems(Startup, setup);

		#[cfg(feature = "debugging")]
		app.add_plugins(bevy_xpbd_3d::plugins::PhysicsDebugPlugin::default())
			.insert_gizmo_group(
				bevy_xpbd_3d::prelude::PhysicsGizmos::default(),
				GizmoConfig {
					enabled: false,
					..default()
				},
			)
			.add_systems(Update, (toggle_projection, toggle_phys_gizmos));
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct GlobalsScene(pub Handle<DynamicScene>);

pub fn setup(
	mut cmds: Commands,
	assets: Res<AssetServer>,
	mut scene_spawner: ResMut<SceneSpawner>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
) {
	let globals_scene = assets.load("globals.scn.ron");
	cmds.insert_resource(GlobalsScene(globals_scene.clone()));
	scene_spawner.spawn_dynamic(globals_scene);

	cmds.spawn((DirectionalLightBundle { ..default() },));
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(100.0))),
			material: mats.add(Color::WHITE),
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(100.0, 100.0, 100.0),
	));
}

#[cfg(feature = "debugging")]
pub fn toggle_projection(mut q: Query<&mut Projection>, keys: Res<ButtonInput<KeyCode>>) {
	if keys.just_pressed(KeyCode::KeyO) {
		for mut proj in &mut q {
			let new = match &*proj {
				Projection::Perspective(_) => {
					Projection::Orthographic(OrthographicProjection::default())
				}
				Projection::Orthographic(_) => {
					Projection::Perspective(PerspectiveProjection::default())
				}
			};
			*proj = new;
		}
	}
}

#[cfg(feature = "debugging")]
pub fn toggle_phys_gizmos(mut store: ResMut<GizmoConfigStore>, keys: Res<ButtonInput<KeyCode>>) {
	if keys.just_pressed(KeyCode::KeyG) {
		let (gizmos, _phys) = store.config_mut::<bevy_xpbd_3d::prelude::PhysicsGizmos>();
		gizmos.enabled = !gizmos.enabled;
	}
}
