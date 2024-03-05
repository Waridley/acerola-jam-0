use crate::{cam::CamPlugin, happens::HappeningsPlugin, player::PlayerPlugin};
use bevy::{
	pbr::{CascadeShadowConfigBuilder, NotShadowCaster},
	prelude::*,
};
use bevy_xpbd_3d::{
	plugins::PhysicsPlugins,
	prelude::{Collider, Gravity, RigidBody},
};
use data::DataPlugin;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_6};
use time_graph::TimeGraphPlugin;

pub mod cam;
pub mod data;
pub mod happens;
pub mod player;
pub mod time_graph;

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

		app.insert_resource(Gravity(Vec3::NEG_Z * 9.81));

		// Mine
		app.add_plugins((
			DataPlugin,
			CamPlugin,
			HappeningsPlugin,
			TimeGraphPlugin,
			PlayerPlugin,
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

	cmds.spawn((DirectionalLightBundle {
		directional_light: DirectionalLight {
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_6)),
		cascade_shadow_config: CascadeShadowConfigBuilder {
			num_cascades: 1,
			minimum_distance: 50.0,
			maximum_distance: 80.0,
			..default()
		}
		.into(),
		..default()
	},));
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: mats.add(Color::WHITE),
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
	));
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: mats.add(Color::WHITE),
			transform: Transform {
				translation: Vec3::new(1.0, 0.0, -0.5),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
	));
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: mats.add(Color::WHITE),
			transform: Transform {
				translation: Vec3::new(2.0, 0.0, -0.75),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
	));
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: mats.add(Color::WHITE),
			transform: Transform {
				translation: Vec3::new(3.0, 0.0, -0.875),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
	));
	let panel_col = Collider::cuboid(16.0, 16.0, 1.0);
	let panel_mesh = meshes.add(Cuboid::new(16.0, 16.0, 1.0));
	let dark_gray = mats.add(Color::DARK_GRAY);
	cmds.spawn((
		PbrBundle {
			mesh: panel_mesh.clone(),
			material: dark_gray.clone(),
			transform: Transform::from_translation(Vec3::NEG_Z * 1.0),
			..default()
		},
		RigidBody::Static,
		panel_col.clone(),
		NotShadowCaster,
	));
	cmds.spawn((
		PbrBundle {
			mesh: panel_mesh.clone(),
			material: dark_gray.clone(),
			transform: Transform {
				translation: Vec3::new(0.0, 8.5, 6.5),
				rotation: Quat::from_rotation_x(FRAC_PI_2),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		panel_col.clone(),
		NotShadowCaster,
	));
	cmds.spawn((
		PbrBundle {
			mesh: panel_mesh.clone(),
			material: dark_gray.clone(),
			transform: Transform {
				translation: Vec3::new(-7.5, 0.0, 6.5),
				rotation: Quat::from_rotation_y(FRAC_PI_2),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		panel_col.clone(),
		NotShadowCaster,
	));
	cmds.spawn((
		PbrBundle {
			mesh: panel_mesh.clone(),
			material: dark_gray.clone(),
			transform: Transform {
				translation: Vec3::new(7.5, 0.0, 6.5),
				rotation: Quat::from_rotation_y(FRAC_PI_2),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		panel_col.clone(),
		NotShadowCaster,
	));
	cmds.spawn((
		TransformBundle {
			local: Transform {
				translation: Vec3::new(0.0, -8.5, 6.5),
				rotation: Quat::from_rotation_x(FRAC_PI_2),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		panel_col.clone(),
		NotShadowCaster,
	));
	// Orb
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Sphere::new(0.3)),
			material: mats.add(Color::ORANGE_RED),
			transform: Transform::from_translation(Vec3::Z * 0.8),
			..default()
		},
		Collider::sphere(0.3),
		RigidBody::Static,
	));
}

#[cfg(feature = "debugging")]
pub fn toggle_projection(mut q: Query<&mut Projection>, keys: Res<ButtonInput<KeyCode>>) {
	if keys.just_pressed(KeyCode::KeyO) {
		for mut proj in &mut q {
			let new = match &*proj {
				Projection::Perspective(_) => Projection::Orthographic(cam::ortho_projection()),
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
