use crate::{
	data::{
		cam::AvoidOccludingPlayer,
		sprites::{LoadSprite3d, Sprite3dBundle},
		tl::{DoList, ReflectDo, Trigger, TriggerKind},
		LoadStdMat,
	},
	happens::TakeBranch,
	scn::{clock::ClockScene, Resettable},
};
use bevy::{
	ecs::system::{Command},
	pbr::NotShadowCaster,
	prelude::*,
};
use bevy_xpbd_3d::{
	components::{*},
	parry::shape::SharedShape,
	prelude::{Collider, Sensor, },
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::f32::consts::FRAC_PI_2;

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
	srv: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
	mut animations: ResMut<Assets<AnimationClip>>,
	clock: Res<ClockScene>,
) {
	fn panel_trigger() -> impl Bundle {
		(
			Trigger {
				oneshot: true,
				causes: DoList(vec![Box::new(OpenPanel)]),
				kind: TriggerKind::Interact {
					message: "Break into panel".into(),
				},
			},
			TransformBundle::from_transform(Transform {
				translation: Vec3::new(0.7, 2.0, 0.0),
				rotation: Quat::from_rotation_z(FRAC_PI_2),
				..default()
			}),
			VisibilityBundle::default(),
			Sensor,
			Collider::cuboid(0.8, 0.2, 0.8),
			Resettable::default(),
		)
	}
	cmds.spawn(panel_trigger());

	let white = mats.add(Color::WHITE);
	cmds.spawn((
		PbrBundle {
			mesh: meshes.add(Cuboid::from_size(Vec3::splat(1.0))),
			material: white.clone(),
			transform: Transform::from_translation(Vec3::Y * 2.0),
			..default()
		},
		RigidBody::Static,
		Collider::cuboid(1.0, 1.0, 1.0),
	));
	let Sprite3dBundle { pbr, .. } = LoadSprite3d {
		size: Vec2::ONE,
		material: LoadStdMat {
			base_color_texture: Some("scn/intro/hack_panel.png".into()),
			double_sided: true,
			cull_mode: None,
			..default()
		},
		transform: Transform {
			translation: Vec3::new(0.52, 2.0, 0.0),
			rotation: Quat::from_rotation_z(FRAC_PI_2),
			..default()
		},
		..default()
	}
	.load_using(&srv, &mut meshes, &mut mats, &mut atlas_layouts);

	{
		fn panel_bundle(pbr: PbrBundle) -> impl Bundle {
			(
				pbr.clone(),
				HackablePanel,
				LockedAxes::ALL_LOCKED,
				RigidBody::Dynamic,
				Collider::round_cuboid(0.8, 0.01, 0.8, 0.05),
				Resettable::new(move |id, world: &mut World| {
					world.entity_mut(id).despawn();
					world.spawn(panel_trigger());
					world.spawn(panel_bundle(pbr));
				}),
			)
		}
		cmds.spawn(panel_bundle(pbr));
	}

	cmds.spawn((
		IntroClock,
		SceneBundle {
			scene: clock.0.clone(),
			transform: Transform {
				translation: Vec3::new(0.0, 2.0, 1.05),
				..default()
			},
			..default()
		},
		RigidBody::Dynamic,
		LockedAxes::ALL_LOCKED,
		Collider::cylinder(0.2, 0.5),
	));

	let panel_mesh = meshes.add(Cuboid::new(12.0, 12.0, 1.0));
	let panel_col = Collider::cuboid(12.0, 12.0, 1.0);
	let dark_gray = mats.add(Color::rgb(0.1, 0.1, 0.1));

	cmds.spawn((
		PbrBundle {
			mesh: panel_mesh.clone(),
			material: mats.add(Color::rgb(0.01, 0.01, 0.01)),
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
		Resettable::new(|id, world: &mut World| {
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

#[derive(Copy, Clone, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
pub struct OpenPanel;

#[derive(Component, Debug)]
pub struct HackablePanel;

impl Command for OpenPanel {
	fn apply(self, world: &mut World) {
		let mut q =
			world.query_filtered::<(&mut LockedAxes, &mut Transform), With<HackablePanel>>();
		let (mut axes, mut xform) = q.single_mut(world);
		*axes = LockedAxes::new();
		xform.translation.y -= 0.02;
		xform.rotation *= Quat::from_rotation_x(-0.05);

		world.spawn((
			Trigger {
				oneshot: true,
				causes: DoList(vec![
					Box::new(TakeBranch("tl/area_1.tl.ron".into())),
					Box::new(BreakClock),
				]),
				kind: TriggerKind::Interact {
					message: "Hack Detonator".into(),
				},
			},
			TransformBundle::from_transform(Transform {
				translation: Vec3::new(0.7, 2.0, 0.0),
				rotation: Quat::from_rotation_z(FRAC_PI_2),
				..default()
			}),
			VisibilityBundle::default(),
			Sensor,
			Collider::cuboid(0.8, 0.2, 0.8),
			Resettable::default(),
		));
	}
}

#[derive(Copy, Clone, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
pub struct BreakClock;

#[derive(Component)]
pub struct IntroClock;

impl Command for BreakClock {
	fn apply(self, world: &mut World) {
		let mut q = world.query_filtered::<&mut LockedAxes, With<IntroClock>>();
		let mut clock = q.single_mut(world);
		*clock = LockedAxes::new();
	}
}
