use bevy::prelude::*;
use bevy_xpbd_3d::parry::shape::SharedShape;
use bevy_xpbd_3d::prelude::{Collider, Sensor};
use serde::{Deserialize, Serialize};
use crate::data::tl::{Do, ReflectDo, InsertPortalTo, TPath};

pub struct HappeningsPlugin;

impl Plugin for HappeningsPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<SpawnPortalTo>();
	}
}

#[derive(Component, Debug, Default, Reflect, Serialize, Deserialize)]
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
struct ReflectBall {
	radius: f32,
}

impl Default for ReflectBall {
	fn default() -> Self {
		Self {
			radius: 0.5,
		}
	}
}

impl From<ReflectBall> for SharedShape {
	fn from(value: ReflectBall) -> Self {
		Self::ball(value.radius)
	}
}

impl Do for SpawnPortalTo {
	fn apply(&self, mut cmds: Commands) {
		cmds.spawn((
			Collider::from(SharedShape::from(self.sensor)),
			self.transform,
			self.global_transform,
			Sensor,
		))
			.add(InsertPortalTo(self.target.clone()));
	}
}
