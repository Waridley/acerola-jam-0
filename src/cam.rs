use bevy::prelude::*;
use std::f32::consts::FRAC_1_SQRT_2;

pub struct CamPlugin;

impl Plugin for CamPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup);
	}
}

pub fn setup(mut cmds: Commands) {
	cmds.spawn(TransformBundle::from_transform(Transform {
		translation: Vec3::new(0.0, -500.0, 500.0),
		rotation: Quat::from_rotation_arc(
			Vec3::NEG_Z,
			Vec3::new(0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
		),
		..default()
	}))
	.with_children(|cmds| {
		cmds.spawn((Camera3dBundle {
			camera: Camera {
				hdr: true,
				..default()
			},
			projection: Projection::Orthographic(OrthographicProjection::default()),
			..default()
		},));
	});
}
