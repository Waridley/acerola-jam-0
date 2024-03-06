use crate::data::tl::TimeLoop;
use bevy::{pbr::NotShadowCaster, prelude::*};
use std::f32::consts::{FRAC_PI_2, TAU};

#[derive(Resource)]
pub struct ClockScene(pub Handle<Scene>);

impl FromWorld for ClockScene {
	fn from_world(world: &mut World) -> Self {
		let mut scn = World::new();

		let base_mesh = world.resource_mut::<Assets<Mesh>>().add(Circle::new(0.25));
		let base_mat = world
			.resource_mut::<Assets<StandardMaterial>>()
			.add(Color::rgb(0.64, 0.64, 0.64));
		scn.spawn((PbrBundle {
			mesh: base_mesh,
			material: base_mat,
			transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
			..default()
		},));

		let mut rim = Torus::new(0.2, 0.3).mesh().minor_resolution(3).build();
		rim.duplicate_vertices();
		rim.compute_flat_normals();
		let rim_mesh = world.resource_mut::<Assets<Mesh>>().add(rim);
		let rim_mat = world
			.resource_mut::<Assets<StandardMaterial>>()
			.add(Color::rgb(0.3, 0.3, 0.7));
		scn.spawn((
			PbrBundle {
				mesh: rim_mesh,
				material: rim_mat,
				..default()
			},
			NotShadowCaster,
		));

		let hand_mesh = world
			.resource_mut::<Assets<Mesh>>()
			.add(Cuboid::new(0.05, 0.05, 0.175));
		let hand_mat = world
			.resource_mut::<Assets<StandardMaterial>>()
			.add(Color::BLACK);
		scn.spawn((
			TransformBundle::default(),
			VisibilityBundle::default(),
			Hand,
		))
		.with_children(|scn| {
			scn.spawn((
				PbrBundle {
					mesh: hand_mesh,
					material: hand_mat,
					transform: Transform::from_translation(Vec3::Z * 0.0875),
					..default()
				},
				NotShadowCaster,
			));
		});

		let handle = world
			.resource_mut::<Assets<Scene>>()
			.add(Scene { world: scn });

		Self(handle)
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Hand;

pub fn tick_hand(mut q: Query<&mut Transform, With<Hand>>, loop_time: Res<TimeLoop>) {
	for mut xform in &mut q {
		xform.rotation =
			Quat::IDENTITY * (Quat::from_rotation_y(-TAU * loop_time.curr.1.as_secs_f32() / 60.0));
	}
}
