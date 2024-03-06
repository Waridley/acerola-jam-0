use crate::data::{
	tl::{AssetServerExt, MomentRef, PortalTo, ReflectDo, TPath, Timeline},
	Str,
};
use bevy::{asset::AssetPath, ecs::system::Command, prelude::*, utils::HashMap};
use bevy_xpbd_3d::{
	parry::shape::SharedShape,
	prelude::{Collider, Sensor},
};
use serde::{Deserialize, Serialize};

pub struct HappeningsPlugin;

impl Plugin for HappeningsPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<SpawnPortalTo>()
			.register_type::<ModifyTimeline>()
			.register_type::<Despawn>()
			.register_type::<ResetLoop>();
	}
}

#[derive(Component, Debug, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[serde(default)]
#[type_path = "happens"]
pub struct SpawnPortalTo {
	pub target: TPath,
	pub sensor: ReflectBall,
	pub transform: Transform,
	pub global_transform: GlobalTransform,
}

#[derive(Component, Debug, Copy, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[serde(default)]
#[type_path = "happens"]
pub struct ReflectBall {
	pub radius: f32,
}

impl Default for ReflectBall {
	fn default() -> Self {
		Self { radius: 30.0 }
	}
}

impl From<ReflectBall> for SharedShape {
	fn from(value: ReflectBall) -> Self {
		Self::ball(value.radius)
	}
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
			Collider::from(SharedShape::from(self.sensor)),
			self.transform,
			self.global_transform,
			Sensor,
			PortalTo(t),
		));
	}
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[type_path = "happens"]
pub struct SpawnScene {
	pub scene: AssetPath<'static>,
}

impl Command for SpawnScene {
	fn apply(self, world: &mut World) {
		let handle = world.resource::<AssetServer>().load(self.scene);
		world.resource_mut::<SceneSpawner>().spawn(handle);
	}
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[type_path = "happens"]
pub struct SpawnDynamicScene {
	pub scene: AssetPath<'static>,
}

impl Command for SpawnDynamicScene {
	fn apply(self, world: &mut World) {
		let handle = world.resource::<AssetServer>().load(self.scene);
		world.resource_mut::<SceneSpawner>().spawn_dynamic(handle);
	}
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize, Do)]
#[type_path = "happens"]
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
			for update in command.timeline_updates {
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
	pub timeline_updates: Vec<MomentUpdate>,
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
	entity: EntityPath,
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
		world.despawn(id);
	}
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct ResetLoop {}

impl Command for ResetLoop {
	fn apply(self, _world: &mut World) {
		todo!()
	}
}
