use crate::{
	data::{
		phys::ColliderShape,
		tl::{
			AssetServerExt, LoadedTimelines, MomentRef, PortalTo, ReflectDo, TPath, TimeLoop,
			Timeline, Trigger,
		},
		Str,
	},
	scn::{spawn_environment, Resettable},
};
use bevy::{
	asset::{AssetPath, UntypedAssetId},
	ecs::system::{Command, RunSystemOnce},
	prelude::*,
	utils::HashMap,
};
use bevy_xpbd_3d::prelude::{Collider, Sensor};
use serde::{Deserialize, Serialize};

pub struct HappeningsPlugin;

impl Plugin for HappeningsPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<SpawnTrigger>()
			.register_type::<SpawnPortalTo>()
			.register_type::<SpawnScene>()
			.register_type::<SpawnDynamicScene>()
			.register_type::<ModifyTimeline>()
			.register_type::<Despawn>()
			.register_type::<ResetLoop>();
	}
}

#[derive(Component, Debug, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[serde(default)]
#[type_path = "happens"]
pub struct TakeBranch(AssetPath<'static>);

impl Command for TakeBranch {
	fn apply(self, world: &mut World) {
		let path = self.0.clone();
		let Some(id) = world
			.resource::<AssetServer>()
			.get_path_id(self.0)
			.map(UntypedAssetId::typed)
		else {
			error!("No `AssetId` for {path}");
			return;
		};
		world.resource_mut::<TimeLoop>().curr.0 = id;
	}
}

#[derive(Component, Default, Clone, Reflect, Deserialize)]
#[reflect(Do, Deserialize)]
#[serde(default)]
#[type_path = "happens"]
pub struct SpawnTrigger {
	pub trigger: Trigger,
	pub sensor: ColliderShape,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
}

impl Command for SpawnTrigger {
	fn apply(self, world: &mut World) {
		world.spawn((
			TransformBundle {
				local: self.transform,
				global: self.global_transform,
			},
			Collider::from(self.sensor.clone()),
			Sensor,
			self.trigger,
			Resettable,
		));
	}
}

#[derive(Component, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[serde(default)]
#[type_path = "happens"]
pub struct SpawnPortalTo {
	pub target: TPath,
	pub sensor: ColliderShape,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
}

impl Command for SpawnPortalTo {
	fn apply(self, world: &mut World) {
		let Some(t) = world
			.resource::<AssetServer>()
			.t_for_t_path(self.target.clone())
		else {
			error!("Timeline {} is not loaded", &self.target.0);
			return;
		};
		world.spawn((
			Collider::from(self.sensor.clone()),
			self.transform,
			self.global_transform,
			Sensor,
			PortalTo(t),
			Resettable,
		));
	}
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct SpawnScene {
	pub path: AssetPath<'static>,
}

impl Command for SpawnScene {
	fn apply(self, world: &mut World) {
		let handle = world.resource::<AssetServer>().load(self.path);
		world.resource_mut::<SceneSpawner>().spawn(handle);
	}
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct SpawnDynamicScene {
	pub path: AssetPath<'static>,
}

impl Command for SpawnDynamicScene {
	fn apply(self, world: &mut World) {
		let handle = world.resource::<AssetServer>().load(self.path);
		world.resource_mut::<SceneSpawner>().spawn_dynamic(handle);
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

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct ResetLoop {}

impl Command for ResetLoop {
	fn apply(self, world: &mut World) {
		let timelines = world.resource::<LoadedTimelines>();
		let srv = world.resource::<AssetServer>();
		for path in timelines.keys() {
			srv.reload(path)
		}
		let mut tloop = world.resource_mut::<TimeLoop>();
		tloop.curr.1 = default();
		let mut q = world.query_filtered::<Entity, With<Resettable>>();
		let mut to_despawn = Vec::new();
		for id in q.iter(world) {
			to_despawn.push(id);
		}
		for id in to_despawn {
			world.entity_mut(id).despawn_recursive();
		}
		world.run_system_once(spawn_environment);
	}
}
