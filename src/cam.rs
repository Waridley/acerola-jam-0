use crate::{data::cam::CamAnchor, player::IsPlayer};
use bevy::{
	prelude::*, render::camera::ScalingMode, transform::TransformSystem::TransformPropagate,
};
use bevy_xpbd_3d::PhysicsSet;
use std::f32::consts::FRAC_1_SQRT_2;

pub struct CamPlugin;

impl Plugin for CamPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup).add_systems(
			PostUpdate,
			cam_follow_player
				.after(PhysicsSet::Sync)
				.before(TransformPropagate),
		);
	}
}

pub fn setup(mut cmds: Commands) {
	cmds.spawn((TransformBundle::default(), CamAnchor))
		.with_children(|cmds| {
			cmds.spawn(TransformBundle::from_transform(Transform {
				translation: Vec3::new(0.0, -10.0, 10.0),
				rotation: Quat::from_rotation_arc(
					// Default camera view direction
					Vec3::NEG_Z,
					// Desired view direction
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
					projection: Projection::Orthographic(ortho_projection()),
					..default()
				},));
			});
		});
}

pub fn cam_follow_player(
	mut cams: Query<&mut Transform, With<CamAnchor>>,
	players: Query<&Transform, (IsPlayer, Without<CamAnchor>)>,
) {
	let mut cam = cams.single_mut();
	let player = players.single();

	cam.translation = player.translation;
}

#[inline]
pub fn ortho_projection() -> OrthographicProjection {
	OrthographicProjection {
		// Top of screen must see (0.0, 500.0, -500.0)
		// Camera is placed at (0.0, -500.0, 500.0)
		//
		far: 3000.0,
		scaling_mode: ScalingMode::FixedVertical(18.0),
		..default()
	}
}
