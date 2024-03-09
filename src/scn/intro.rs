use crate::data::tl::ReflectDo;
use bevy::{ecs::system::Command, pbr::NotShadowCaster, prelude::*};
use bevy_xpbd_3d::{components::RigidBody, prelude::Collider};
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_2;

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<FlipLever>().add_systems(Startup, setup);
	}
}

pub fn setup(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
) {
	let panel_mesh = meshes.add(Cuboid::new(12.0, 12.0, 1.0));
	let panel_col = Collider::cuboid(12.0, 12.0, 1.0);
	let dark_gray = mats.add(Color::DARK_GRAY);

	cmds.spawn((
		PbrBundle {
			mesh: panel_mesh.clone(),
			material: dark_gray.clone(),
			transform: Transform::from_translation(Vec3::NEG_Z),
			..default()
		},
		RigidBody::Static,
		panel_col.clone(),
		NotShadowCaster,
	));

	cmds.spawn((
		Name::new("Walls"),
		TransformBundle::default(),
		VisibilityBundle::default(),
	))
	.with_children(|cmds| {
		cmds.spawn((
			PbrBundle {
				mesh: panel_mesh.clone(),
				material: dark_gray.clone(),
				transform: Transform {
					translation: Vec3::new(0.0, 6.5, 4.5),
					rotation: Quat::from_rotation_x(FRAC_PI_2),
					..default()
				},
				..default()
			},
			RigidBody::Static,
			panel_col.clone(),
			NotShadowCaster,
		));
		cmds.spawn((
			PbrBundle {
				mesh: panel_mesh.clone(),
				material: dark_gray.clone(),
				transform: Transform {
					translation: Vec3::new(-5.5, 0.0, 4.5),
					rotation: Quat::from_rotation_y(FRAC_PI_2),
					..default()
				},
				..default()
			},
			RigidBody::Static,
			panel_col.clone(),
			NotShadowCaster,
		));
		cmds.spawn((
			PbrBundle {
				mesh: panel_mesh.clone(),
				material: dark_gray.clone(),
				transform: Transform {
					translation: Vec3::new(5.5, 0.0, 4.5),
					rotation: Quat::from_rotation_y(FRAC_PI_2),
					..default()
				},
				..default()
			},
			RigidBody::Static,
			panel_col.clone(),
			NotShadowCaster,
		));
		cmds.spawn((
			TransformBundle {
				local: Transform {
					translation: Vec3::new(0.0, -6.5, 4.5),
					rotation: Quat::from_rotation_x(FRAC_PI_2),
					..default()
				},
				..default()
			},
			RigidBody::Static,
			panel_col.clone(),
			NotShadowCaster,
		));
	});
}

#[derive(Copy, Clone, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
pub struct FlipLever;

impl Command for FlipLever {
	fn apply(self, world: &mut World) {
		let mut levers = world.query::<(&Name, &mut TextureAtlas)>();
		let Some((_, mut atlas)) = levers
			.iter_mut(world)
			.find(|(name, _)| &***name == "IntroLever")
		else {
			error!("Failed to find IntroLever");
			return;
		};
		atlas.index = if atlas.index == 0 { 1 } else { 0 }
	}
}
