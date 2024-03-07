use crate::{data::cam::CamAnchor, player::IsPlayer};
use bevy::{
	prelude::*, render::camera::ScalingMode, transform::TransformSystem::TransformPropagate,
};
use bevy_xpbd_3d::PhysicsSet;

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
	let translation = Vec3::new(0.0, -40.0, 20.0);
	cmds.spawn((TransformBundle::default(), CamAnchor))
		.with_children(|cmds| {
			cmds.spawn(TransformBundle::from_transform(Transform {
				translation,
				rotation: Quat::from_rotation_arc(
					// Default camera view direction
					Vec3::NEG_Z,
					// Desired view direction
					-translation.normalize(),
				),
				..default()
			}))
			.with_children(|cmds| {
				cmds.spawn((Camera3dBundle {
					camera: Camera {
						hdr: true,
						clear_color: ClearColorConfig::Custom(Color::BLACK),
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
	let Ok(player) = players.get_single() else {
		return;
	};

	cam.translation = player.translation;
}

#[inline]
pub fn ortho_projection() -> OrthographicProjection {
	OrthographicProjection {
		scaling_mode: ScalingMode::FixedVertical(9.0),
		far: 200.0,
		..default()
	}
}
