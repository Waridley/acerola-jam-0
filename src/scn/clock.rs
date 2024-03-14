use crate::{
	data::{
		sprites::{LoadAtlas3d, LoadSprite3d, Sprite3dBundle},
		tl::TimeLoop,
		LoadAlphaMode, LoadStdMat,
	},
	scn::clock::hand::HandItem,
	GameState,
};
use bevy::{ecs::system::RunSystemOnce, pbr::light_consts::lux::AMBIENT_DAYLIGHT, prelude::*};
use sond_bevy_enum_components::{
	EntityEnumCommands, EntityWorldEnumMut, EnumComponent, WithVariant,
};
use std::f32::consts::{FRAC_PI_2, TAU};

pub struct ClockPlugin;

impl Plugin for ClockPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup.after(crate::cam::setup))
			.add_systems(
				Update,
				fade_clock_on_reset.run_if(in_state(GameState::ResettingLoop)),
			);
	}

	fn finish(&self, app: &mut App) {
		app.init_resource::<ClockScene>();
	}
}

pub fn setup(
	mut cmds: Commands,
	srv: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
	cam: Query<Entity, WithVariant<crate::data::cam::cam_node::Gimbal>>,
) {
	let cam = cam.single();
	let bundle = LoadSprite3d {
		size: Vec2::splat(8.0),
		material: LoadStdMat {
			base_color_texture: Some("scn/clock/clock_bg.png".into()),
			base_color: Color::rgba(1.0, 1.0, 1.0, 0.0),
			unlit: true,
			alpha_mode: LoadAlphaMode::Blend,
			..default()
		},
		transform: Transform {
			translation: Vec3::NEG_Z * 2.0,
			rotation: Quat::from_rotation_x(-FRAC_PI_2),
			..default()
		},
		..default()
	}
	.load_using(&srv, &mut meshes, &mut mats, &mut atlas_layouts);

	let clock = cmds
		.spawn((bundle.pbr, FullscreenClock))
		.with_children(|cmds| {
			let Sprite3dBundle {
				atlas: Some((meshes, _)),
				pbr,
			} = LoadSprite3d {
				size: Vec2::new(1.0, 4.0),
				anchor: Vec2::new(0.0, -1.6),
				material: LoadStdMat {
					base_color_texture: Some("scn/clock/hands.png".into()),
					base_color: Color::rgba(0.0, 0.0, 0.0, 0.0),
					unlit: true,
					alpha_mode: LoadAlphaMode::Multiply,
					..default()
				},
				atlas_layout: Some(LoadAtlas3d {
					tile_size: Vec2::new(128.0, 1024.0),
					columns: 2,
					rows: 1,
					padding: None,
					offset: None,
				}),
				..default()
			}
			.load_using(&srv, &mut meshes, &mut mats, &mut atlas_layouts)
			else {
				unreachable!()
			};
			let [hour_mesh, minute_mesh] = meshes.0.try_into().expect("exactly 2 tiles");
			cmds.spawn((
				PbrBundle {
					mesh: hour_mesh,
					transform: Transform::from_translation(Vec3::NEG_Y * 0.1),
					..pbr.clone()
				},
				FullscreenClock,
			))
			.with_enum(hand::Hour);
			cmds.spawn((
				PbrBundle {
					mesh: minute_mesh,
					transform: Transform::from_translation(Vec3::NEG_Y * 0.2),
					..pbr.clone()
				},
				FullscreenClock,
			))
			.with_enum(hand::Minute);
		})
		.id();

	cmds.entity(cam).add_child(clock);
}

#[derive(Resource)]
pub struct ClockScene(pub Handle<Scene>);

impl FromWorld for ClockScene {
	fn from_world(world: &mut World) -> Self {
		let mut scn = World::new();

		let bundle = world.run_system_once_with(
			LoadSprite3d {
				size: Vec2::splat(1.0),
				material: LoadStdMat {
					base_color_texture: Some("scn/clock/clock_bg_small.png".into()),
					alpha_mode: LoadAlphaMode::Blend,
					double_sided: true,
					cull_mode: None,
					..default()
				},
				..default()
			},
			LoadSprite3d::loader_system,
		);
		scn.spawn(bundle.pbr);

		let Sprite3dBundle {
			atlas: Some((meshes, _)),
			pbr,
		} = world.run_system_once_with(
			LoadSprite3d {
				size: Vec2::new(0.125, 0.5),
				atlas_layout: Some(LoadAtlas3d {
					tile_size: Vec2::new(16.0, 128.0),
					columns: 2,
					rows: 1,
					padding: None,
					offset: None,
				}),
				material: LoadStdMat {
					base_color_texture: Some("scn/clock/hands_small.png".into()),
					base_color: Color::rgba(0.0, 0.0, 0.0, 1.1),
					alpha_mode: LoadAlphaMode::Multiply,
					double_sided: true,
					cull_mode: None,
					..default()
				},
				anchor: Vec2::new(0.0, -0.18),
				..default()
			},
			LoadSprite3d::loader_system,
		)
		else {
			unreachable!()
		};
		let [hour_mesh, minute_mesh] = meshes.0.try_into().expect("should have exactly 2 meshes");
		scn.spawn(PbrBundle {
			mesh: hour_mesh,
			transform: Transform::from_translation(Vec3::NEG_Y * 0.01),
			..pbr.clone()
		})
		.with_enum(hand::Hour);
		scn.spawn(PbrBundle {
			mesh: minute_mesh,
			transform: Transform::from_translation(Vec3::NEG_Y * 0.02),
			..pbr
		})
		.with_enum(hand::Minute);

		let handle = world
			.resource_mut::<Assets<Scene>>()
			.add(Scene { world: scn });

		Self(handle)
	}
}

#[derive(EnumComponent)]
#[component(mutable, derive(Reflect))]
pub enum Hand {
	Hour,
	Minute,
}

pub fn tick_hand(mut q: Query<(&mut Transform, Hand)>, loop_time: Res<TimeLoop>) {
	for (mut xform, hand) in &mut q {
		match hand {
			HandItem::Hour(_) => {
				xform.rotation = Quat::IDENTITY
					* (Quat::from_rotation_y(TAU * loop_time.curr.1.secs_f32() / 3600.0))
			}
			HandItem::Minute(_) => {
				xform.rotation = Quat::IDENTITY
					* (Quat::from_rotation_y(TAU * loop_time.curr.1.secs_f32() / 60.0))
			}
		}
	}
}

#[derive(Component)]
pub struct FullscreenClock;

pub fn fade_clock_on_reset(
	tloop: Res<TimeLoop>,
	q: Query<(&Handle<StandardMaterial>, Option<Hand>), With<FullscreenClock>>,
	mut lights: Query<&mut DirectionalLight>,
	mut ambient_light: ResMut<AmbientLight>,
	mut mats: ResMut<Assets<StandardMaterial>>,
) {
	let TimeLoop {
		curr,
		resetting_from: from,
		resetting_to: to,
	} = *tloop;
	let t = if to < from {
		let range = from - to;
		1.0 - ((from - curr.1).secs_f32() / range.secs_f32())
	} else {
		let range = to - from;
		(curr.1 - from).secs_f32() / range.secs_f32()
	};
	let t = 1.0 - ((t - 0.5).abs() * 2.0);
	let t = (t * 8.0).clamp(0.0, 1.0);
	for (handle, hand) in &q {
		let Some(mat) = mats.get_mut(handle) else {
			error!("missing material for {handle:?}");
			continue;
		};
		let t = if hand.is_some() { t * 1.2 } else { t };
		mat.base_color = mat.base_color.with_a(t);
	}
	for mut light in &mut lights {
		light.illuminance = AMBIENT_DAYLIGHT * (1.0 - t);
	}
	ambient_light.brightness = 80.0 * (1.0 - t);
}
