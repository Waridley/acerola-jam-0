use crate::{
	data::{
		sprites::{LoadAtlas3d, LoadSprite3d, Sprite3dBundle},
		tl::TimeLoop,
		LoadAlphaMode, LoadStdMat,
	},
	scn::clock::hand::HandItem,
};
use bevy::{ecs::system::RunSystemOnce, prelude::*};
use sond_bevy_enum_components::{EntityWorldEnumMut, EnumComponent};
use std::f32::consts::{TAU};

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
