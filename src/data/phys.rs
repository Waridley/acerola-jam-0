use bevy::prelude::*;
use bevy_xpbd_3d::{parry::shape::SharedShape, prelude::Collider};
use serde::{Deserialize, Serialize};

pub struct PhysDataPlugin;

impl Plugin for PhysDataPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(First, insert_collider_shapes)
			.add_systems(PreUpdate, insert_collider_shapes)
			.add_systems(Update, insert_collider_shapes)
			.add_systems(PostUpdate, insert_collider_shapes)
			.add_systems(Last, insert_collider_shapes);
	}
}

#[derive(Reflect, Component, Clone, Serialize, Deserialize)]
#[reflect(Default, Component, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColliderShape(#[reflect(ignore)] SharedShape);

impl Default for ColliderShape {
	fn default() -> Self {
		Self(SharedShape::ball(0.5))
	}
}

pub fn insert_collider_shapes(mut cmds: Commands, q: Query<(Entity, &ColliderShape)>) {
	for (id, shape) in &q {
		cmds.entity(id)
			.insert(Collider::from(shape.clone()))
			.remove::<ColliderShape>();
	}
}

impl From<ColliderShape> for Collider {
	fn from(value: ColliderShape) -> Self {
		Self::from(value.0)
	}
}
