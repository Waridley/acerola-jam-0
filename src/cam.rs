use crate::{
	data::cam::{
		cam_node::{Anchor, Gimbal, WithoutCamNode},
		AvoidOccludingPlayer,
	},
	player::player_entity,
};
use bevy::{
	prelude::*, render::camera::ScalingMode, transform::TransformSystem::TransformPropagate,
};
use bevy_xpbd_3d::{parry::math::Point, PhysicsSet};
use sond_bevy_enum_components::{EntityEnumCommands, WithVariant};

pub fn cam_resting_pos() -> Transform {
	let translation = Vec3::new(0.0, -40.0, 20.0);
	Transform {
		translation,
		rotation: Quat::from_rotation_arc(
			// Default camera view direction
			Vec3::NEG_Z,
			// Desired view direction
			-translation.normalize(),
		),
		..default()
	}
}

pub struct CamPlugin;

impl Plugin for CamPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup).add_systems(
			PostUpdate,
			(
				cam_follow_player
					.after(PhysicsSet::Sync)
					.before(TransformPropagate),
				dont_occlude_player,
			),
		);
		#[cfg(feature = "debugging")]
		app.add_systems(Update, move_cam);
	}
}

pub fn setup(mut cmds: Commands) {
	cmds.insert_resource(Msaa::Off);
	cmds.spawn(TransformBundle::default())
		.with_enum(Anchor)
		.with_children(|cmds| {
			cmds.spawn(TransformBundle::from_transform(cam_resting_pos()))
				.with_enum(Gimbal)
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
	mut cams: Query<&mut Transform, WithVariant<Anchor>>,
	players: Query<&Transform, (WithVariant<player_entity::Root>, WithoutCamNode)>,
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

#[cfg(feature = "debugging")]
pub fn move_cam(
	keys: Res<ButtonInput<KeyCode>>,
	mut anchor_q: Query<&mut Transform, WithVariant<Anchor>>,
	mut gimbal_q: Query<&mut Transform, WithVariant<Gimbal>>,
	mut proj_q: Query<&mut Projection>,
	t: Res<Time>,
) {
	let dt = t.delta_seconds();

	if keys.just_pressed(KeyCode::KeyP) {
		let mut proj = proj_q.single_mut();
		let new = match &*proj {
			Projection::Perspective(_) => Projection::Orthographic(ortho_projection()),
			Projection::Orthographic(_) => {
				Projection::Perspective(PerspectiveProjection::default())
			}
		};
		*proj = new;
	}

	let mut anchor = anchor_q.single_mut();
	let mut gimbal = gimbal_q.single_mut();
	if keys.pressed(KeyCode::Semicolon) {
		anchor.rotation = Quat::IDENTITY;
		*gimbal = cam_resting_pos();
		return;
	}

	let mut offset = Vec3::ZERO;
	if keys.pressed(KeyCode::KeyI) {
		offset.y += 1.0;
	}
	if keys.pressed(KeyCode::KeyK) {
		offset.y -= 1.0;
	}
	if keys.pressed(KeyCode::KeyU) {
		offset.z += 1.0;
	}
	if keys.pressed(KeyCode::KeyO) {
		offset.z -= 1.0;
	}

	if offset.length() > 0.2 {
		let mut new = gimbal.translation + offset * dt * 8.0;
		new.y = f32::min(-0.5, new.y);
		gimbal.translation = new;
		gimbal.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, -new.normalize());
	}

	if keys.pressed(KeyCode::KeyJ) {
		anchor.rotation *= Quat::from_rotation_z(-dt);
	}
	if keys.pressed(KeyCode::KeyL) {
		anchor.rotation *= Quat::from_rotation_z(dt);
	}
}

pub fn dont_occlude_player(
	player_q: Query<&GlobalTransform, WithVariant<player_entity::Root>>,
	mut q: Query<(&GlobalTransform, &mut Visibility, &mut AvoidOccludingPlayer)>,
) {
	let player = player_q.single();
	for (xform, mut vis, mut test) in &mut q {
		// TODO: use raycasting? Or a shader?
		if let Some(shape) = test.shape() {
			let point: Point<f32> = player.reparented_to(xform).translation.into();
			if *vis == Visibility::Hidden {
				if !shape.contains_local_point(&Point::new(
					point.x - (0.1 * point.x.signum()),
					point.y - (0.1 * point.y.signum()),
					point.z - 0.1 * point.z.signum(),
				)) {
					*vis = Visibility::Visible
				}
		} else if shape.contains_local_point(&Point::new(
				point.x + (0.1 * point.x.signum()),
				point.y + (0.1 * point.y.signum()),
				point.z + 0.1 * point.z.signum(),
			)) {
				*vis = Visibility::Hidden
			}
		} else if *vis == Visibility::Hidden {
			// Hysteresis
			if player.translation().y < xform.translation().y - 0.1 {
				*vis = Visibility::Visible
			}
		} else {
			// Hysteresis
			if player.translation().y > xform.translation().y + 0.1 {
				*vis = Visibility::Hidden
			}
		};
	}
}
