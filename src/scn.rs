use crate::scn::clock::ClockScene;
use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_xpbd_3d::{components::RigidBody, prelude::Collider};
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_2;

pub mod clock;
pub mod intro;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
	fn build(&self, _app: &mut App) {}

	fn finish(&self, app: &mut App) {
		app.init_resource::<ClockScene>();
	}
}

#[derive(Component, Copy, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EnvRoot;

pub fn spawn_environment(
	mut cmds: Commands,
	mut scene_spawner: ResMut<SceneSpawner>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	clock: Res<ClockScene>,
) {
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: mats.add(Color::WHITE),
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
		EnvRoot,
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
		EnvRoot,
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
		EnvRoot,
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
		EnvRoot,
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
		EnvRoot,
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
		EnvRoot,
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
		EnvRoot,
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
		EnvRoot,
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
		EnvRoot,
	));

	cmds.spawn((
		Name::new("Orb"),
		PbrBundle {
			mesh: meshes.add(Sphere::new(0.3)),
			material: mats.add(Color::ORANGE_RED),
			transform: Transform::from_translation(Vec3::Z * 0.8),
			..default()
		},
		Collider::sphere(0.3),
		RigidBody::Static,
		EnvRoot,
	));

	let clock_anchor = cmds.spawn((
		TransformBundle::from_transform(Transform::from_translation(Vec3::NEG_Y * 0.6)),
		VisibilityBundle::default(),
		EnvRoot,
	));
	scene_spawner.spawn_as_child(clock.0.clone(), clock_anchor.id());
}
