use bevy::prelude::*;
use bevy_tnua::{
	controller::{TnuaController, TnuaControllerPlugin},
	prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaControllerBundle},
	TnuaProximitySensor,
};
use bevy_tnua_xpbd3d::{TnuaXpbd3dPlugin, TnuaXpbd3dSensorShape};
use bevy_xpbd_3d::{
	parry::shape::SharedShape,
	prelude::{Collider, RigidBody},
};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_2;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			TnuaControllerPlugin,
			TnuaXpbd3dPlugin,
			InputManagerPlugin::<Action>::default(),
		))
		.add_systems(Startup, setup)
		.add_systems(Update, move_player);
	}
}

pub type PlayerTag = TnuaController;
pub type IsPlayer = With<PlayerTag>;

pub fn setup(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
) {
	let input_map = InputMap::new([
		(Action::Move, UserInput::from(VirtualDPad::wasd())),
		(Action::Move, VirtualDPad::arrow_keys().into()),
		(Action::Move, DualAxis::left_stick().into()),
		(Action::Jump, GamepadButtonType::South.into()),
		(Action::Jump, KeyCode::Space.into()),
		(Action::Dash, GamepadButtonType::RightTrigger2.into()),
	]);

	cmds.spawn((
		TransformBundle::from_transform(Transform::from_translation(Vec3::Z * 4.0)),
		RigidBody::Dynamic,
		Collider::from(SharedShape::capsule_z(0.5, 0.25)),
		TnuaControllerBundle {
			motor: Default::default(),
			rigid_body_tracker: Default::default(),
			proximity_sensor: TnuaProximitySensor {
				cast_origin: Vec3::NEG_Z * 0.5,
				cast_direction: Direction3d::NEG_Z,
				..default()
			},
			..default()
		},
		TnuaXpbd3dSensorShape(Collider::sphere(0.24)),
		InputManagerBundle::with_map(input_map),
	))
	.with_children(|cmds| {
		cmds.spawn(PbrBundle {
			mesh: meshes.add(
				Capsule3d {
					radius: 0.25,
					half_length: 0.625,
				}
				.mesh(),
			),
			material: mats.add(Color::SALMON),
			transform: Transform {
				translation: Vec3::NEG_Z * 0.125,
				rotation: Quat::from_rotation_x(FRAC_PI_2),
				..default()
			},
			..default()
		});
		cmds.spawn(PbrBundle {
			mesh: meshes.add(
				Capsule3d {
					radius: 0.1,
					half_length: 0.2,
				}
				.mesh(),
			),
			material: mats.add(Color::SALMON),
			transform: Transform {
				translation: Vec3::new(-0.25, 0.0, 0.0),
				rotation: Quat::from_rotation_x(FRAC_PI_2),
				..default()
			},
			..default()
		});
		cmds.spawn(PbrBundle {
			mesh: meshes.add(
				Capsule3d {
					radius: 0.1,
					half_length: 0.2,
				}
				.mesh(),
			),
			material: mats.add(Color::SALMON),
			transform: Transform {
				translation: Vec3::new(0.25, 0.0, 0.0),
				rotation: Quat::from_rotation_x(FRAC_PI_2),
				..default()
			},
			..default()
		});
	});
}

#[derive(Debug, Actionlike, Copy, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum Action {
	Move,
	Jump,
	Dash,
}

pub fn move_player(mut q: Query<(&mut TnuaController, &ActionState<Action>)>) {
	for (mut ctrl, state) in &mut q {
		let v = state
			.clamped_axis_pair(&Action::Move)
			.map_or(Vec2::ZERO, |data| data.xy() * 4.0);

		ctrl.basis(TnuaBuiltinWalk {
			desired_velocity: Vec3::new(v.x, v.y, 0.0),
			up: Direction3d::Z,
			float_height: 0.25,
			cling_distance: 0.05,
			acceleration: 24.0,
			air_acceleration: 8.0,
			..default()
		});

		if state.pressed(&Action::Jump) {
			ctrl.action(TnuaBuiltinJump {
				height: 2.0,
				..default()
			});
		}
	}
}
