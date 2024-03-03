use bevy::prelude::*;
use bevy_sprite3d::{Sprite3d, Sprite3dParams, Sprite3dPlugin};
use bevy_tnua::{
	controller::{TnuaController, TnuaControllerPlugin},
	prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaControllerBundle},
	TnuaProximitySensor,
};
use bevy_tnua_xpbd3d::{TnuaXpbd3dPlugin, TnuaXpbd3dSensorShape};
use bevy_xpbd_3d::{
	parry::shape::SharedShape,
	prelude::{Collider, LockedAxes, RigidBody},
};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use std::time::Duration;
use sond_bevy_enum_components::{EntityEnumCommands, EnumComponent, WithVariant};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			TnuaControllerPlugin,
			TnuaXpbd3dPlugin,
			InputManagerPlugin::<Action>::default(),
			Sprite3dPlugin,
		))
		.add_systems(
			Update,
			(
				move_player,
				animate_player,
				spawn_player.run_if(resource_exists::<PlayerSpriteSheet>),
			),
		);
		let assets = app.world.resource::<AssetServer>();
		let id = assets.load("player.png");
		app.insert_resource(PlayerSpriteSheet(id));
	}
}

pub type PlayerTag = TnuaController;
pub type IsPlayer = With<PlayerTag>;

#[derive(EnumComponent, Copy, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum PlayerEntity {
	Root,
	Sprite,
}

#[derive(Resource)]
pub struct PlayerSpriteSheet(pub Handle<Image>);

pub fn spawn_player(
	mut cmds: Commands,
	assets: Res<AssetServer>,
	mut params: Sprite3dParams,
	sheet: Res<PlayerSpriteSheet>,
) {
	if !assets.is_loaded_with_dependencies(sheet.0.clone()) {
		return;
	}

	let input_map = InputMap::new([
		(Action::Move, UserInput::from(VirtualDPad::wasd())),
		(Action::Move, VirtualDPad::arrow_keys().into()),
		(Action::Move, DualAxis::left_stick().into()),
		(Action::Jump, GamepadButtonType::South.into()),
		(Action::Jump, KeyCode::Space.into()),
		(Action::Jump, KeyCode::Backspace.into()),
		(Action::Dash, GamepadButtonType::RightTrigger2.into()),
	]);

	let layout = TextureAtlasLayout::from_grid(Vec2::new(256.0, 512.0), 4, 2, None, None);
	let layout = params.atlas_layouts.add(layout);

	cmds.spawn((
		TransformBundle {
			local: Transform::from_translation(Vec3::Z * 2.0),
			..default()
		},
		VisibilityBundle::default(),
		RigidBody::Dynamic,
		Collider::from(SharedShape::capsule_z(0.125, 0.25)),
		TnuaControllerBundle {
			motor: Default::default(),
			rigid_body_tracker: Default::default(),
			proximity_sensor: TnuaProximitySensor {
				cast_origin: Vec3::NEG_Z * 0.125,
				cast_direction: Direction3d::NEG_Z,
				..default()
			},
			..default()
		},
		TnuaXpbd3dSensorShape(Collider::sphere(0.2)),
		InputManagerBundle::with_map(input_map),
		LockedAxes::ROTATION_LOCKED,
	))
	.with_enum(player_entity::Root)
	.with_children(|cmds| {
		cmds.spawn((
			Sprite3d {
				transform: Transform {
					translation: Vec3 {
						x: 0.0,
						// Nudge towards camera to align feet with far edge of platforms before falling off.
						y: -0.125,
						// Nudge down to compensate for half of float height.
						z: -0.125,
					},
					// Z is up.
					rotation: Quat::from_rotation_x(FRAC_PI_2),
					..default()
				},
				image: sheet.0.clone(),
				alpha_mode: AlphaMode::Blend,
				pixels_per_metre: 512.0,
				..default()
			}
			.bundle_with_atlas(&mut params, TextureAtlas { layout, index: 0 }),
			PlayerAnimationState {
				timer: Timer::new(Duration::from_millis(300), TimerMode::Repeating),
				curr_animation: default()
			},
		)).with_enum(player_entity::Sprite);
	});
	cmds.remove_resource::<PlayerSpriteSheet>();
}

#[derive(Debug, Actionlike, Copy, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum Action {
	Move,
	Jump,
	Dash,
}

pub fn move_player(
	mut q: Query<(Entity, &mut TnuaController, &ActionState<Action>)>,
	mut anim_q: Query<(&mut PlayerAnimationState, &Parent)>,
) {
	for (id, mut ctrl, action_state) in &mut q {
		let v = action_state
			.clamped_axis_pair(&Action::Move)
			.map_or(Vec2::ZERO, |data| data.xy() * 2.0);

		ctrl.basis(TnuaBuiltinWalk {
			desired_velocity: Vec3::new(v.x, v.y, 0.0),
			up: Direction3d::Z,
			float_height: 0.375,
			cling_distance: 0.05,
			acceleration: 24.0,
			air_acceleration: 8.0,
			spring_strengh: 1200.0,
			spring_dampening: 0.5,
			..default()
		});

		for (mut anim_state, parent) in &mut anim_q {
			if parent.get() == id {
				use PlayerAnimation::*;
				if v.angle_between(Vec2::Y).abs() < FRAC_PI_4 {
					anim_state.curr_animation = Forward
				} else if v.angle_between(Vec2::NEG_Y).abs() < FRAC_PI_4 {
					anim_state.curr_animation = Backward
				};
			}
		}
		if action_state.pressed(&Action::Jump) {
			ctrl.action(TnuaBuiltinJump {
				height: 1.2,
				takeoff_extra_gravity: 5.0,
				fall_extra_gravity: 10.0,
				shorten_extra_gravity: 10.0,
				..default()
			});
		}
	}
}

pub fn animate_player(
	mut q: Query<(&mut TextureAtlas, &mut PlayerAnimationState), WithVariant<player_entity::Sprite>>,
	t: Res<Time>,
) {
	for (mut atlas, mut state) in &mut q {
		state.timer.tick(t.delta());
		let new = if state.timer.just_finished() {
			atlas.index + 1
		} else {
			atlas.index
		};
		let new = (new % 4) + (state.curr_animation as usize * 4);
		if atlas.index != new {
			atlas.index = new;
		}
	}
}

#[derive(Component)]
pub struct PlayerAnimationState {
	pub timer: Timer,
	pub curr_animation: PlayerAnimation,
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(usize)]
pub enum PlayerAnimation {
	#[default]
	Backward = 0,
	Forward = 1,
	Left = 2,
	Right = 3,
}