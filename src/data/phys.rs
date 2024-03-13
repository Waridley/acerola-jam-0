use bevy::prelude::*;
use bevy_xpbd_3d::{
	parry::{na::Unit, shape::SharedShape},
	prelude::Collider,
};
use serde::{Deserialize, Serialize};

pub struct PhysDataPlugin;

impl Plugin for PhysDataPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<ColliderShape>()
			.add_systems(First, insert_collider_shapes)
			.add_systems(PreUpdate, insert_collider_shapes)
			.add_systems(Update, insert_collider_shapes)
			.add_systems(PostUpdate, insert_collider_shapes)
			.add_systems(Last, insert_collider_shapes);
	}
}

#[derive(Reflect, Component, Clone, Debug, Serialize, Deserialize)]
#[reflect(Default, Component, Serialize, Deserialize)]
pub enum ColliderShape {
	Ball {
		radius: f32,
	},
	Cuboid {
		x: f32,
		y: f32,
		z: f32,
	},
	Capsule {
		a: Vec3,
		b: Vec3,
		radius: f32,
	},
	Segment {
		a: Vec3,
		b: Vec3,
	},
	Triangle {
		a: Vec3,
		b: Vec3,
		c: Vec3,
	},
	TriMesh {},
	Polyline {},
	HalfSpace {
		normal: Vec3,
	},
	HeightField {},
	Compound {},
	ConvexPolyhedron {},
	Cylinder {
		half_height: f32,
		radius: f32,
	},
	Cone {
		half_height: f32,
		radius: f32,
	},
	RoundCuboid {
		x: f32,
		y: f32,
		z: f32,
		border_radius: f32,
	},
	RoundTriangle {
		a: Vec3,
		b: Vec3,
		c: Vec3,
		border_radius: f32,
	},
	RoundCylinder {
		half_height: f32,
		radius: f32,
		border_radius: f32,
	},
	RoundCone {
		half_height: f32,
		radius: f32,
		border_radius: f32,
	},
	RoundConvexPolyhedron {},
	Custom {},
}

impl Default for ColliderShape {
	fn default() -> Self {
		Self::Ball { radius: 0.5 }
	}
}

pub fn insert_collider_shapes(mut cmds: Commands, q: Query<(Entity, &ColliderShape)>) {
	for (id, shape) in &q {
		cmds.entity(id)
			.insert(Collider::from(shape.clone()))
			.remove::<ColliderShape>();
	}
}

impl From<ColliderShape> for SharedShape {
	fn from(value: ColliderShape) -> Self {
		use ColliderShape::*;
		match value {
			Ball { radius } => SharedShape::ball(radius),
			Cuboid { x, y, z } => SharedShape::cuboid(x * 0.5, y * 0.5, z * 0.5),
			Capsule { a, b, radius } => SharedShape::capsule(a.into(), b.into(), radius),
			Segment { a, b } => SharedShape::segment(a.into(), b.into()),
			Triangle { a, b, c } => SharedShape::triangle(a.into(), b.into(), c.into()),
			TriMesh {} => todo!(),
			Polyline {} => todo!(),
			HalfSpace { normal } => SharedShape::halfspace(Unit::new_normalize(normal.into())),
			HeightField {} => todo!(),
			Compound {} => todo!(),
			ConvexPolyhedron {} => todo!(),
			Cylinder {
				half_height,
				radius,
			} => SharedShape::cylinder(half_height, radius),
			Cone {
				half_height,
				radius,
			} => SharedShape::cone(half_height, radius),
			RoundCuboid {
				x,
				y,
				z,
				border_radius,
			} => SharedShape::round_cuboid(x * 0.5, y * 0.5, z * 0.5, border_radius),
			RoundTriangle {
				a,
				b,
				c,
				border_radius,
			} => SharedShape::round_triangle(a.into(), b.into(), c.into(), border_radius),
			RoundCylinder {
				half_height,
				radius,
				border_radius,
			} => SharedShape::round_cylinder(half_height, radius, border_radius),
			RoundCone {
				half_height,
				radius,
				border_radius,
			} => SharedShape::round_cone(half_height, radius, border_radius),
			RoundConvexPolyhedron {} => todo!(),
			Custom {} => todo!(),
		}
	}
}

impl From<ColliderShape> for Collider {
	fn from(value: ColliderShape) -> Self {
		Self::from(SharedShape::from(value))
	}
}
