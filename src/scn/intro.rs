use crate::{
	data::{cam::AvoidOccludingPlayer, tl::ReflectDo},
	scn::Resettable,
};
use bevy::{
	ecs::system::{Command, EntityCommands},
	pbr::NotShadowCaster,
	prelude::*,
};
use bevy_xpbd_3d::{components::RigidBody, parry::shape::SharedShape, prelude::Collider};
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_2;
use parking_lot::RwLock;

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<RaiseWalls>()
			.register_type::<FlipLever>()
			.add_systems(Startup, setup);
	}
}

pub fn setup(
	mut cmds: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut animations: ResMut<Assets<AnimationClip>>,
) {
	let panel_mesh = meshes.add(Cuboid::new(12.0, 12.0, 1.0));
	let panel_col = Collider::cuboid(12.0, 12.0, 1.0);
	let dark_gray = mats.add(Color::rgb(0.05, 0.05, 0.05));

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
		Walls,
		Name::new("Walls"),
		TransformBundle::from_transform(Transform::from_translation(Vec3::Z * -8.0)),
		VisibilityBundle::default(),
		Resettable::new(|mut cmds: EntityCommands| {
			cmds.add(|id, world: &mut World| {
				let mut entity = world.entity_mut(id);
				entity
					.get_mut::<Transform>()
					.expect("Player definitely has a Transform")
					.translation
					.z = -8.0;
				let mut player = entity
					.get_mut::<AnimationPlayer>()
					.expect("Walls should have an AnimationPlayer");
				player.replay();
				player.pause();
			});
		}),
		AnimationPlayer::default(),
	))
	.with_children(|cmds| {
		cmds.spawn((
			PbrBundle {
				mesh: panel_mesh.clone(),
				material: dark_gray.clone(),
				transform: Transform {
					translation: Vec3::new(0.0, 6.5, 0.0),
					rotation: Quat::from_rotation_x(FRAC_PI_2),
					..default()
				},
				..default()
			},
			RigidBody::Static,
			panel_col.clone(),
			NotShadowCaster,
			AvoidOccludingPlayer {
				area_shape: RwLock::new(Some(SharedShape::cuboid(6.0, 6.0, 2.5))),
				area_transform: Transform::from_translation(Vec3::NEG_Z * 2.5),
				..default()
			},
		));
		cmds.spawn((
			PbrBundle {
				mesh: panel_mesh.clone(),
				material: dark_gray.clone(),
				transform: Transform {
					translation: Vec3::new(-5.5, 0.0, 0.0),
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
					translation: Vec3::new(5.5, 0.0, 0.0),
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
					translation: Vec3::new(0.0, -6.5, 0.0),
					rotation: Quat::from_rotation_x(FRAC_PI_2),
					..default()
				},
				visibility: Visibility::Hidden,
				..default()
			},
			RigidBody::Static,
			panel_col.clone(),
			NotShadowCaster,
			AvoidOccludingPlayer {
				area_shape: RwLock::new(Some(SharedShape::cuboid(6.0, 6.0, 2.5))),
				area_transform: Transform::from_translation(Vec3::NEG_Z * 2.5),
				..default()
			},
		));
	});

	let mut clip = AnimationClip::default();
	clip.add_curve_to_path(
		EntityPath {
			parts: vec!["Walls".into()],
		},
		VariableCurve {
			keyframe_timestamps: vec![0.0, 0.5],
			keyframes: Keyframes::Translation(vec![Vec3::Z * -8.0, Vec3::Z * -4.0]),
			interpolation: Interpolation::Linear,
		},
	);
	let raise_walls = animations.add(clip);
	cmds.insert_resource(RaiseWallsClip(raise_walls));
}

#[derive(Component)]
pub struct Walls;

#[derive(Copy, Clone, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
pub struct RaiseWalls;

#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct RaiseWallsClip(pub Handle<AnimationClip>);

impl Command for RaiseWalls {
	fn apply(self, world: &mut World) {
		let clip = world.resource::<RaiseWallsClip>().0.clone();
		let mut q = world.query_filtered::<&mut AnimationPlayer, With<Walls>>();
		q.single_mut(world).play(clip).resume();
	}
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
