use crate::scn::{clock::ClockPlugin, intro::IntroPlugin};
use bevy::{
	ecs::system::{EntityCommand, EntityCommands},
	pbr::CascadeShadowConfigBuilder,
	prelude::*,
};
use bevy_xpbd_3d::prelude::Collider;
use serde::{Deserialize, Serialize};
use sond_bevy_enum_components::reflect::AppEnumReflectExt;
use std::f32::consts::FRAC_PI_6;

pub mod clock;
pub mod intro;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
	fn build(&self, app: &mut App) {
		app.register_variant::<clock::hand::Hour>()
			.register_variant::<clock::hand::Minute>()
			.register_type::<Resettable>()
			.add_systems(Startup, setup)
			.add_plugins((IntroPlugin, ClockPlugin));
	}
}

pub fn setup(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
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
			material: mats.add(Color::rgb(0.15, 0.2, 0.1)),
			transform: Transform::from_translation(Vec3::NEG_Z * 1.5),
			..default()
		},
		Collider::halfspace(Vec3::Z),
	));
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
	pub fn new(resetter: impl EntityCommand + Clone + Sync + 'static) -> Self {
		Self {
			resetter: Box::new(move |mut cmds: EntityCommands| {
				cmds.add(resetter.clone());
			}),
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
