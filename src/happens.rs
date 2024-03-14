use crate::{
	data::{
		phys::ColliderShape,
		tl::{
			Lifetime, LoadedTimelines, LoopTime, MomentRef, ReflectDo, SpawnedAt, TimeLoop,
			Timeline, Trigger,
		},
		Str,
	},
	player::player_entity::Root,
	scn::Resettable,
};
use bevy::{
	asset::{AssetPath, UntypedAssetId},
	ecs::system::{Command, CommandQueue},
	prelude::*,
	utils::HashMap,
};
use bevy_xpbd_3d::prelude::{Collider, Sensor};
use serde::{Deserialize, Serialize};
use sond_bevy_enum_components::WithVariant;

pub struct HappeningsPlugin;

impl Plugin for HappeningsPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<TakeBranch>()
			.register_type::<SpawnTrigger>()
			.register_type::<SpawnScene>()
			.register_type::<SpawnDynamicScene>()
			.register_type::<ModifyTimeline>()
			.register_type::<Despawn>()
			.register_type::<MovePlayerTo>()
			.register_type::<ResetLoop>();
	}
}

#[derive(Component, Debug, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[serde(default)]
#[type_path = "happens"]
pub struct TakeBranch(pub AssetPath<'static>);

impl Command for TakeBranch {
	fn apply(self, world: &mut World) {
		let branch_path = self.0.clone();
		// let curr_id = world.resource::<TimeLoop>().curr.0;
		// let timelines = world.resource::<Assets<Timeline>>();
		// let curr_tl = timelines
		// 	.get(curr_id)
		// 	.expect("Can't be missing the current timeline");
		let srv = world.resource::<AssetServer>();
		let Some(branch_id) = srv
			.get_path_id(branch_path.clone())
			.map(UntypedAssetId::typed)
		else {
			error!("No `AssetId` for {branch_path}");
			return;
		};
		// let Some(tl) = timelines.get(branch_id) else {
		// 	error!("Timeline {branch_path} is not yet in the `Assets` resource");
		// 	return;
		// };

		// TODO: To truly validate branches, would have to follow all branch/merge paths
		//  until certain there's no way the new branch could be reached from the
		//  current timeline.

		// if tl.branch_from.map(|t| t.0) != Some(curr_id)
		// && curr_tl.branch_from.map(|t| t.0) != Some(branch_id) {
		// 	error!("Timeline {branch_path} does not branch from the current timeline");
		// 	return;
		// };

		world.resource_mut::<TimeLoop>().curr.0 = branch_id;
	}
}

#[derive(Component, Default, Clone, Reflect, Deserialize)]
#[reflect(Do, Deserialize)]
#[serde(default)]
#[type_path = "happens"]
pub struct SpawnTrigger {
	pub name: Option<Name>,
	pub trigger: Trigger,
	pub sensor: ColliderShape,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
	pub lifetime: Option<LoopTime>,
}

impl Command for SpawnTrigger {
	fn apply(self, world: &mut World) {
		let timestamp = SpawnedAt(world.resource::<TimeLoop>().curr.1);
		let mut cmds = world.spawn((
			TransformBundle {
				local: self.transform,
				global: self.global_transform,
			},
			Collider::from(self.sensor.clone()),
			Sensor,
			self.trigger,
			Resettable::default(),
			timestamp,
		));
		if let Some(name) = self.name {
			cmds.insert(name);
		}
		if let Some(lt) = self.lifetime {
			cmds.insert(Lifetime(lt));
		}
	}
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct SpawnScene {
	pub path: AssetPath<'static>,
	#[serde(default)]
	pub transform: Transform,
}

impl Command for SpawnScene {
	fn apply(self, world: &mut World) {
		let handle = world.resource::<AssetServer>().load(self.path);
		world.spawn(SceneBundle {
			scene: handle,
			transform: self.transform,
			..default()
		});
	}
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct SpawnDynamicScene {
	pub path: AssetPath<'static>,
	#[serde(default)]
	pub transform: Transform,
}

impl Command for SpawnDynamicScene {
	fn apply(self, world: &mut World) {
		let handle = world.resource::<AssetServer>().load(self.path);
		world.spawn(DynamicSceneBundle {
			scene: handle,
			transform: self.transform,
			..default()
		});
	}
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize, Do)]
#[type_path = "happens"]
#[serde(transparent)]
pub struct ModifyTimeline(Vec<TimelineCommand>);

impl Command for ModifyTimeline {
	fn apply(self, world: &mut World) {
		for command in self.0 {
			let srv = world.resource::<AssetServer>();
			let Some(id) = srv.get_path_id(command.path.clone()) else {
				error!("Missing timeline for {}", &command.path);
				continue;
			};
			let id = id.typed::<Timeline>();
			let mut timelines = world.resource_mut::<Assets<Timeline>>();
			let Some(tl) = timelines.get_mut(id) else {
				error!("Timeline {id} not found");
				continue;
			};
			for update in command.updates {
				let Some(moment) = tl.get_moment_mut(&update.moment) else {
					error!("Moment {:?} not found", &update.moment);
					continue;
				};
				if let Some(setter) = update.disabled {
					setter.apply_to(&mut moment.disabled)
				}
				for (label, update) in update.happenings {
					let Some(happenings) = moment
						.happenings
						.iter_mut()
						.find(|happenings| happenings.label.as_deref() == Some(&label))
					else {
						error!("Happenings {label} not found");
						continue;
					};
					update.apply_to(&mut happenings.disabled);
				}
			}
		}
	}
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[type_path = "happens"]
pub struct TimelineCommand {
	pub path: AssetPath<'static>,
	pub updates: Vec<MomentUpdate>,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[type_path = "happens"]
pub struct MomentUpdate {
	pub moment: MomentRef,
	#[serde(default)]
	pub disabled: Option<SetDisabled>,
	#[serde(default)]
	pub happenings: HashMap<Str, SetDisabled>,
}

#[derive(Debug, Copy, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[type_path = "happens"]
pub enum SetDisabled {
	Set { disabled: bool },
	Toggle,
}

impl SetDisabled {
	pub fn apply_to(self, flag: &mut bool) {
		match self {
			SetDisabled::Set { disabled } => *flag = disabled,
			SetDisabled::Toggle => *flag = !*flag,
		}
	}
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct Despawn {
	#[serde(with = "crate::data::entity_path_str")]
	pub entity: EntityPath,
	#[serde(default = "_true")]
	recursive: bool,
}

fn _true() -> bool {
	true
}

impl Command for Despawn {
	fn apply(self, world: &mut World) {
		let mut q = world.query::<(Entity, &Name, Option<&Children>)>();
		let mut parts = self.entity.parts.into_iter();
		let Some(mut curr) = parts.next() else {
			error!("entity path is empty");
			return;
		};
		let Some((mut id, _, mut children)) = q.iter(world).find(|(_, name, _)| *name == &curr)
		else {
			error!("Missing entity {curr}");
			return;
		};
		for name in parts {
			let Some(kids) = children else {
				error!("Entity {curr} has no children");
				return;
			};
			curr = name.clone();
			for child in kids.into_iter().copied() {
				if let Ok(child) = q.get(world, child) {
					if *child.1 == name {
						id = child.0;
						children = child.2;
						break;
					}
				}
			}
		}
		if self.recursive {
			world.entity_mut(id).despawn_recursive();
		} else {
			world.despawn(id);
		}
	}
}

#[derive(Default, Debug, Copy, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
#[serde(transparent)]
pub struct MovePlayerTo(pub Vec3);

impl Command for MovePlayerTo {
	fn apply(self, world: &mut World) {
		let mut q = world.query_filtered::<&mut Transform, WithVariant<Root>>();
		let mut player = q.single_mut(world);
		player.translation = self.0;
	}
}

#[derive(Default, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
#[serde(default)]
pub struct ResetLoop {
	pub to: LoopTime,
}

impl Command for ResetLoop {
	fn apply(self, world: &mut World) {
		let timelines = world.resource::<LoadedTimelines>();
		let srv = world.resource::<AssetServer>();
		for path in timelines.keys() {
			srv.reload(path)
		}
		let mut tloop = world.resource_mut::<TimeLoop>();
		tloop.curr.1 = self.to;
		let mut q = world.query::<(Entity, &Resettable)>();
		let mut queue = CommandQueue::default();
		let mut cmds = Commands::new(&mut queue, &*world);
		for (id, reset) in q.iter(world) {
			let cmds = cmds.entity(id);
			reset.defer_reset(cmds);
		}
		queue.apply(world);
	}
}
