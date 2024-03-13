use crate::{
	data::SystemRegistry,
	scn::{clock::ClockScene, intro::IntroPlugin},
};
use bevy::{ecs::system::EntityCommands, pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_xpbd_3d::{components::RigidBody, prelude::Collider};
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_6;

pub mod clock;
pub mod intro;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<Resettable>()
			.add_systems(Startup, setup)
			.add_plugins(IntroPlugin);
	}

	fn finish(&self, app: &mut App) {
		app.init_resource::<ClockScene>();
	}
}

pub fn setup(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	sys_reg: Res<SystemRegistry>,
) {
	cmds.spawn((DirectionalLightBundle {
		directional_light: DirectionalLight {
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_6)),
		cascade_shadow_config: CascadeShadowConfigBuilder {
			num_cascades: 1,
			maximum_distance: 60.0,
			..default()
		}
		.into(),
		..default()
	},));

	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Plane3d::new(Vec3::Z).mesh().size(1024.0, 1024.0)),
			material: mats.add(Color::rgb(0.1, 0.1, 0.1)),
			transform: Transform::from_translation(Vec3::NEG_Z * 1.5),
			..default()
		},
		Collider::halfspace(Vec3::Z),
	));

	cmds.run_system(sys_reg.spawn_env);
}

#[derive(Component, Reflect, Deref, DerefMut, Serialize, Deserialize)]
#[reflect(Default, Component, Serialize, Deserialize, no_field_bounds)]
#[serde(default)]
pub struct Resettable {
	#[reflect(ignore)]
	#[serde(skip)]
	pub resetter: Box<dyn Resetter>,
}

impl Resettable {
	pub fn new(resetter: impl Resetter + 'static) -> Self {
		Self {
			resetter: Box::new(resetter),
		}
	}
}

impl Default for Resettable {
	fn default() -> Self {
		Self {
			resetter: default_resetter(),
		}
	}
}

pub trait Resetter: Send + Sync {
	fn defer_reset(&self, cmds: EntityCommands);
}

impl<F: Fn(EntityCommands) + Send + Sync> Resetter for F {
	fn defer_reset(&self, cmds: EntityCommands) {
		self(cmds)
	}
}

pub fn default_resetter() -> Box<dyn Resetter> {
	Box::new(despawn_self::<true>)
}

fn despawn_self<const RECURSIVE: bool>(mut cmds: EntityCommands) {
	if RECURSIVE {
		cmds.despawn_recursive();
	} else {
		cmds.despawn();
	}
}

pub fn spawn_environment(
	mut cmds: Commands,
	mut scene_spawner: ResMut<SceneSpawner>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	clock: Res<ClockScene>,
) {
	let white = mats.add(Color::WHITE);
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: white.clone(),
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
		Resettable::default(),
	));
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: white.clone(),
			transform: Transform {
				translation: Vec3::new(1.0, 0.0, -0.5),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
		Resettable::default(),
	));
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: white.clone(),
			transform: Transform {
				translation: Vec3::new(2.0, 0.0, -0.75),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
		Resettable::default(),
	));
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: white.clone(),
			transform: Transform {
				translation: Vec3::new(3.0, 0.0, -0.875),
				..default()
			},
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
		Resettable::default(),
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
		Resettable::default(),
	));

	let clock_anchor = cmds.spawn((
		TransformBundle::from_transform(Transform::from_translation(Vec3::NEG_Y * 0.6)),
		VisibilityBundle::default(),
		Resettable::default(),
	));
	scene_spawner.spawn_as_child(clock.0.clone(), clock_anchor.id());
}
