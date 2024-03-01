use crate::data::tl::{AssetServerExt, PortalTo, ReflectDo, TPath};
use bevy::{asset::AssetPath, ecs::system::Command, prelude::*};
use bevy_xpbd_3d::{
	parry::shape::SharedShape,
	prelude::{Collider, Sensor},
};
use serde::{Deserialize, Serialize};

pub struct HappeningsPlugin;

impl Plugin for HappeningsPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<SpawnPortalTo>();
	}
}

#[derive(Component, Debug, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[serde(default)]
#[type_path = "happens"]
pub struct SpawnPortalTo {
	pub target: TPath,
	pub sensor: ReflectBall,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
}

#[derive(Component, Debug, Copy, Clone, Reflect, Serialize, Deserialize)]
#[serde(default)]
pub struct ReflectBall {
	pub radius: f32,
}

impl Default for ReflectBall {
	fn default() -> Self {
		Self { radius: 30.0 }
	}
}

impl From<ReflectBall> for SharedShape {
	fn from(value: ReflectBall) -> Self {
		Self::ball(value.radius)
	}
}

impl Command for SpawnPortalTo {
	fn apply(self, world: &mut World) {
		let Some(t) = world
			.resource::<AssetServer>()
			.t_for_t_path(self.target.clone())
		else {
			error!("Timeline {} is not loaded", &self.target.0);
			return;
		};
		world.spawn((
			Collider::from(SharedShape::from(self.sensor)),
			self.transform,
			self.global_transform,
			Sensor,
			PortalTo(t),
		));
	}
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[type_path = "happens"]
pub struct SpawnScene {
	pub scene: AssetPath<'static>,
}

impl Command for SpawnScene {
	fn apply(self, world: &mut World) {
		let handle = world.resource::<AssetServer>().load(self.scene);
		world.resource_mut::<SceneSpawner>().spawn(handle);
	}
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[type_path = "happens"]
pub struct SpawnDynamicScene {
	pub scene: AssetPath<'static>,
}

impl Command for SpawnDynamicScene {
	fn apply(self, world: &mut World) {
		let handle = world.resource::<AssetServer>().load(self.scene);
		world.resource_mut::<SceneSpawner>().spawn_dynamic(handle);
	}
}
