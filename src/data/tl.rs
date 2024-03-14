use bevy::{
	asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, BoxedFuture, LoadContext},
	ecs::system::Command,
	prelude::*,
	reflect::{
		serde::TypedReflectDeserializer, List, ListIter, ReflectMut, ReflectOwned, ReflectRef,
		TypeInfo, TypeRegistry, TypeRegistryArc,
	},
	scene::SceneLoaderError,
};
use humantime::DurationError;
use serde::{
	de::{DeserializeSeed, Error, MapAccess, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
	any::Any,
	borrow::Cow,
	collections::BTreeMap,
	fmt::{Debug, Display, Formatter},
	ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign},
	str::FromStr,
	time::Duration,
};

use bevy::utils::{CowArc, HashMap};
use bevy_asset_loader::prelude::*;

use super::Str;
use serde::de::SeqAccess;

pub struct TimeDataPlugin;

#[derive(AssetCollection, Resource)]
pub struct Timelines {
	#[asset(path = "tl/intro.tl.ron")]
	pub intro: Handle<Timeline>,
	#[asset(path = "tl/area_1.tl.ron")]
	pub area_1: Handle<Timeline>,
}

impl Plugin for TimeDataPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<Timeline>()
			.register_type::<Log>()
			.register_type::<LoopTime>()
			.register_type::<T>()
			.register_type::<TPath>()
			.register_type::<Str>()
			.register_type::<MomentRef>()
			.register_type::<TimeLoop>()
			.register_type::<(AssetPath<'static>, T)>()
			.register_type::<PortalPath>()
			.register_type::<PortalTo>()
			.register_type::<Trigger>();
	}

	fn finish(&self, app: &mut App) {
		let registry = app.world.resource::<AppTypeRegistry>();
		let asset_server = app.world.resource::<AssetServer>();
		app.register_asset_loader(TimelineLoader {
			registry: registry.0.clone(),
			asset_server: asset_server.clone(),
		});

		let assets = app.world.resource::<AssetServer>();

		let intro_path = AssetPath::from("tl/intro.tl.ron");
		let intro = assets.load(intro_path.clone());
		let area_path = AssetPath::from("tl/area_1.tl.ron");
		let area_1 = assets.load(area_path.clone());
		app.insert_resource(TimeLoop {
			curr: T(intro.id(), default()),
			resetting_from: default(),
			resetting_to: default(),
		});
		app.insert_resource(LoadedTimelines(
			[(area_path, area_1), (intro_path, intro)].into(),
		));
	}
}

/// Duration since the beginning of the loop
///
/// Serializes in a human-readable format using [humantime]
#[derive(
	Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deref, DerefMut, Reflect,
)]
pub struct LoopTime(i64);

impl Serialize for LoopTime {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&format!("{}", self))
	}
}

impl<'de> Deserialize<'de> for LoopTime {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Self::from_str(<&str as Deserialize<'de>>::deserialize(deserializer)?)
			.map_err(|e| Error::custom(format_args!("{e}")))
	}
}

impl FromStr for LoopTime {
	type Err = DurationError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		fn parse_unsigned(s: &str) -> Result<i64, DurationError> {
			humantime::parse_duration(s).and_then(|dur| {
				dur.as_millis()
					.try_into()
					.map_err(|_| DurationError::NumberOverflow)
			})
		}
		match s.as_bytes()[0] {
			b'-' => parse_unsigned(&s[1..]).map(std::ops::Neg::neg),
			b'+' => parse_unsigned(&s[1..]),
			_ => parse_unsigned(s),
		}
		.map(Self)
	}
}

impl Display for LoopTime {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let sign = if self.0 < 0 { "-" } else { "" };
		let dur = humantime::format_duration(Duration::from_millis(self.0.unsigned_abs()));
		write!(f, "{sign}{dur}",)
	}
}

impl From<f32> for LoopTime {
	fn from(value: f32) -> Self {
		Self(Duration::from_secs_f32(value.abs()).as_millis() as i64 * value.signum() as i64)
	}
}

impl From<i64> for LoopTime {
	fn from(value: i64) -> Self {
		Self(value)
	}
}

impl From<Duration> for LoopTime {
	fn from(value: Duration) -> Self {
		Self(
			value
				.as_millis()
				.try_into()
				.expect("Duration is too long for LoopTime"),
		)
	}
}

impl Add<Duration> for LoopTime {
	type Output = Self;

	fn add(self, rhs: Duration) -> Self::Output {
		Self(self.0 + rhs.as_millis() as i64)
	}
}

impl AddAssign<Duration> for LoopTime {
	fn add_assign(&mut self, rhs: Duration) {
		*self = *self + rhs;
	}
}

impl Sub<Duration> for LoopTime {
	type Output = Self;

	fn sub(self, rhs: Duration) -> Self::Output {
		Self(self.0 - rhs.as_millis() as i64)
	}
}

impl SubAssign<Duration> for LoopTime {
	fn sub_assign(&mut self, rhs: Duration) {
		*self = *self - rhs;
	}
}

impl Add for LoopTime {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl AddAssign for LoopTime {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs
	}
}

impl Sub for LoopTime {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl SubAssign for LoopTime {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

impl LoopTime {
	pub const EPOCH: Self = Self(0);

	pub fn millis(self) -> i64 {
		self.0
	}

	pub fn secs(self) -> i64 {
		self.millis() / 1000
	}

	pub fn secs_f32(self) -> f32 {
		self.millis() as f32 / 1000.0
	}
}

#[derive(Asset, TypePath, Default, Deref, DerefMut)]
pub struct Timeline {
	pub branch_from: Option<T>,
	#[deref]
	pub moments: BTreeMap<LoopTime, Moment>,
	pub merge_into: Option<T>,
}

impl Timeline {
	pub fn get_moment(&self, moment: &MomentRef) -> Option<&Moment> {
		match moment {
			MomentRef::At(t) => self.moments.get(t),
			MomentRef::Labelled(label) => self
				.moments
				.iter()
				.find_map(|(_, mom)| (mom.label.as_deref() == Some(label)).then_some(mom)),
		}
	}

	pub fn get_moment_mut(&mut self, moment: &MomentRef) -> Option<&mut Moment> {
		match moment {
			MomentRef::At(t) => self.moments.get_mut(t),
			MomentRef::Labelled(label) => self
				.moments
				.iter_mut()
				.find_map(|(_, mom)| (mom.label.as_deref() == Some(label)).then_some(mom)),
		}
	}
}

#[derive(Default)]
pub struct Moment {
	pub label: Option<Str>,
	pub desc: Option<CowArc<'static, str>>,
	pub happenings: Vec<Happenings>,
	pub disabled: bool,
}

pub struct Happenings {
	pub label: Option<Str>,
	pub actions: Vec<Box<dyn Do>>,
	pub disabled: bool,
}

#[derive(Reflect, Copy, Clone, Debug, Default)]
pub struct T(pub AssetId<Timeline>, pub LoopTime);

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct TPath(pub AssetPath<'static>, pub LoopTime);

/// Reference to a moment within a timeline.
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum MomentRef {
	At(LoopTime),
	Labelled(Str),
}

impl Default for TPath {
	fn default() -> Self {
		Self("tl/intro.tl.ron".into(), default())
	}
}

impl Index<T> for Assets<Timeline> {
	type Output = Moment;

	fn index(&self, index: T) -> &Self::Output {
		&self
			.get(index.0)
			.unwrap_or_else(|| panic!("Missing Timeline for {}", index.0))
			.moments[&index.1]
	}
}

impl IndexMut<T> for Assets<Timeline> {
	fn index_mut(&mut self, index: T) -> &mut Self::Output {
		self.get_mut(index.0)
			.unwrap_or_else(|| panic!("Missing Timeline for {}", index.0))
			.moments
			.get_mut(&index.1)
			.unwrap_or_else(|| panic!("Missing Moment for time {}", index.1))
	}
}

#[reflect_trait]
pub trait Do: Reflect + Send + Sync {
	fn apply(&self, cmds: Commands);
	fn clone_do(&self) -> Box<dyn Do>;
}

impl<T: Command + Clone + Reflect + Send + Sync> Do for T {
	fn apply(&self, mut cmds: Commands) {
		cmds.add(self.to_owned())
	}

	fn clone_do(&self) -> Box<dyn Do> {
		Box::new(self.clone())
	}
}

/// A dummy happening for debugging time graph code
#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct Log {
	#[serde(default)]
	pub level: LogLevel,
	pub msg: Cow<'static, str>,
}

impl Command for Log {
	fn apply(self, _world: &mut World) {
		match self.level {
			LogLevel::Trace => trace!(target: "happens::Print", "{}", &self.msg),
			LogLevel::Debug => debug!(target: "happens::Print", "{}", &self.msg),
			LogLevel::Info => info!(target: "happens::Print", "{}", &self.msg),
			LogLevel::Warn => warn!(target: "happens::Print", "{}", &self.msg),
			LogLevel::Error => error!(target: "happens::Print", "{}", &self.msg),
		}
	}
}

#[derive(Reflect, Default, Debug, Copy, Clone, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
#[type_path = "happens"]
pub enum LogLevel {
	Trace,
	Debug,
	#[default]
	Info,
	Warn,
	Error,
}

pub struct TimelineLoader {
	pub registry: TypeRegistryArc,
	pub asset_server: AssetServer,
}

impl AssetLoader for TimelineLoader {
	type Asset = Timeline;
	type Settings = ();
	type Error = SceneLoaderError;

	fn load<'a>(
		&'a self,
		reader: &'a mut Reader,
		_settings: &'a Self::Settings,
		_load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
		Box::pin(async move {
			let mut bytes = Vec::new();
			reader.read_to_end(&mut bytes).await?;
			let mut ron_de = ron::de::Deserializer::from_bytes(&bytes)?;
			let tl_de = TimelineDeserializer {
				registry: &self.registry.read(),
				asset_server: &self.asset_server,
			};
			Ok(tl_de
				.deserialize(&mut ron_de)
				.map_err(|e| ron_de.span_error(e))?)
		})
	}

	fn extensions(&self) -> &[&str] {
		&["tl.ron"]
	}
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
pub enum TimelineField {
	BranchFrom,
	Moments,
	MergeInto,
}

pub struct TimelineDeserializer<'a> {
	registry: &'a TypeRegistry,
	asset_server: &'a AssetServer,
}

impl<'de> DeserializeSeed<'de> for TimelineDeserializer<'de> {
	type Value = Timeline;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_struct(
			"Timeline",
			&["moments"],
			TimelineVisitor {
				registry: self.registry,
				asset_server: self.asset_server,
			},
		)
	}
}

pub struct TimelineVisitor<'a> {
	registry: &'a TypeRegistry,
	asset_server: &'a AssetServer,
}

impl<'a, 'de> Visitor<'de> for TimelineVisitor<'a> {
	type Value = Timeline;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
		formatter.write_str("Timeline struct")
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut branch_from = None;
		let mut merge_into = None;
		let mut moments = None;
		while let Some(key) = map.next_key()? {
			match key {
				TimelineField::BranchFrom => {
					branch_from = self.asset_server.t_for_t_path(map.next_value()?)
				}
				TimelineField::Moments => {
					moments = Some(map.next_value_seed(MomentMapDeserializer {
						registry: self.registry,
					})?)
				}
				TimelineField::MergeInto => {
					merge_into = self.asset_server.t_for_t_path(map.next_value()?)
				}
			}
		}
		let moments = moments.ok_or_else(|| Error::missing_field("moments"))?;

		Ok(Timeline {
			branch_from,
			moments,
			merge_into,
		})
	}
}

pub struct MomentMapDeserializer<'a> {
	pub registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for MomentMapDeserializer<'a> {
	type Value = BTreeMap<LoopTime, Moment>;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_map(MomentMapVisitor {
			registry: self.registry,
		})
	}
}

pub struct MomentMapVisitor<'a> {
	pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for MomentMapVisitor<'a> {
	type Value = BTreeMap<LoopTime, Moment>;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
		formatter.write_str("map of LoopTime => Moment")
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut moments = BTreeMap::new();
		while let Some(time) = map.next_key::<LoopTime>()? {
			let moment = map.next_value_seed(MomentDeserializer {
				registry: self.registry,
			})?;

			moments.insert(time, moment);
		}
		Ok(moments)
	}
}

pub struct MomentDeserializer<'a> {
	registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for MomentDeserializer<'a> {
	type Value = Moment;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_struct(
			"Moment",
			&["happenings"],
			MomentVisitor {
				registry: self.registry,
			},
		)
	}
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
pub enum MomentField {
	Label,
	Desc,
	Happenings,
	Disabled,
}

pub struct MomentVisitor<'a> {
	registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for MomentVisitor<'a> {
	type Value = Moment;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
		formatter.write_str("struct Moment")
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut label = None;
		let mut desc = None;
		let mut happenings = None;
		let mut disabled = false;

		while let Some(key) = map.next_key()? {
			match key {
				MomentField::Label => label = Some(map.next_value()?),
				MomentField::Desc => desc = Some(map.next_value::<String>()?.into()),
				MomentField::Happenings => {
					happenings = Some(map.next_value_seed(HappeningsDeserializer {
						registry: self.registry,
					})?)
				}
				MomentField::Disabled => {
					disabled = map.next_value()?;
				}
			}
		}
		Ok(Moment {
			label,
			desc,
			happenings: happenings.unwrap_or_default(),
			disabled,
		})
	}
}

pub struct HappeningsDeserializer<'a> {
	pub registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for HappeningsDeserializer<'a> {
	type Value = Vec<Happenings>;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_seq(HappeningsVisitor {
			registry: self.registry,
		})
	}
}

pub struct HappeningsVisitor<'a> {
	pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for HappeningsVisitor<'a> {
	type Value = Vec<Happenings>;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
		formatter.write_str("A sequence of Happenings")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let mut ret = Vec::new();

		while let Some(happenings) = seq.next_element_seed(HappeningsMapDeserializer {
			registry: self.registry,
		})? {
			ret.push(happenings);
		}

		Ok(ret)
	}
}

pub struct HappeningsMapDeserializer<'a> {
	pub registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for HappeningsMapDeserializer<'a> {
	type Value = Happenings;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_map(HappeningsMapVisitor {
			registry: self.registry,
		})
	}
}
pub struct HappeningsMapVisitor<'a> {
	pub registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for HappeningsMapVisitor<'a> {
	type Value = Happenings;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
		formatter.write_str("a map of `TypePath`s => `Do` implementors")
	}

	fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
	where
		A: MapAccess<'de>,
	{
		let mut actions = Vec::new();
		let mut disabled = false;
		let mut label = None;

		while let Some(key) = map.next_key()? {
			if key == "DISABLED" {
				disabled = map.next_value()?;
				continue;
			}

			if key == "LABEL" {
				label = Some(map.next_value()?);
				continue;
			}

			let reg = self
				.registry
				.get_with_type_path(key)
				.ok_or_else(|| Error::custom(format_args!("No registration found for `{key}`")))?;

			let entry = map.next_value_seed(TypedReflectDeserializer::new(reg, self.registry))?;

			let action = do_from_reflect(entry, self.registry).map_err(|e| {
				Error::custom(format_args!("Failed to downcast {e:?} to `Box<dyn Do>`"))
			})?;

			actions.push(action);
		}

		Ok(Happenings {
			label,
			actions,
			disabled,
		})
	}
}

#[derive(Resource, Debug, Reflect)]
pub struct TimeLoop {
	pub curr: T,
	pub resetting_from: LoopTime,
	pub resetting_to: LoopTime,
}

/// Mainly keeps timeline strong handles alive.
#[derive(Resource, Deref, DerefMut)]
pub struct LoadedTimelines(pub HashMap<AssetPath<'static>, Handle<Timeline>>);

/// Static, \[de]serializable time portal definition
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct PortalPath {
	pub from: TPath,
	pub to: TPath,
}

/// Runtime portal reference.
pub struct Portal {
	pub from: T,
	pub to: T,
}

pub trait AssetServerExt {
	fn t_for_t_path(&self, path: TPath) -> Option<T>;
	fn t_path_for_t(&self, t: T) -> Option<TPath>;
	fn portal_for_portal_path(&self, path: PortalPath) -> Option<Portal>;
	fn portal_path_for_portal(&self, portal: Portal) -> Option<PortalPath>;
}

impl AssetServerExt for AssetServer {
	fn t_for_t_path(&self, path: TPath) -> Option<T> {
		let timeline = self.get_path_id(path.0)?.typed();
		Some(T(timeline, path.1))
	}

	fn t_path_for_t(&self, t: T) -> Option<TPath> {
		let timeline = self.get_path(t.0)?.into_owned();
		Some(TPath(timeline, t.1))
	}

	fn portal_for_portal_path(&self, path: PortalPath) -> Option<Portal> {
		Some(Portal {
			from: self.t_for_t_path(path.from)?,
			to: self.t_for_t_path(path.to)?,
		})
	}

	fn portal_path_for_portal(&self, portal: Portal) -> Option<PortalPath> {
		Some(PortalPath {
			from: self.t_path_for_t(portal.from)?,
			to: self.t_path_for_t(portal.to)?,
		})
	}
}

#[derive(Component, Debug, Reflect)]
pub struct PortalTo(pub T);

#[derive(Component, Reflect, Default, Clone, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Trigger {
	#[serde(default)]
	pub oneshot: bool,
	#[serde(with = "do_list_serde")]
	pub causes: DoList,
	#[serde(default)]
	pub kind: TriggerKind,
}

#[derive(Reflect, Copy, Clone, Default, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum TriggerKind {
	#[default]
	Enter,
	Interact {
		#[serde(default = "crate::data::ui::default_interact_msg")]
		message: Str,
	},
}

pub mod do_list_serde {
	use super::{do_from_reflect, DoList};
	use bevy::{
		reflect::TypeRegistry,
		scene::serde::{SceneMapDeserializer, SceneMapSerializer},
	};
	use serde::{
		de::{DeserializeSeed, Error},
		Deserializer, Serialize, Serializer,
	};

	pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DoList, D::Error> {
		let registry = crate::type_registry().read();
		DoListDeserializer {
			registry: &registry,
		}
		.deserialize(deserializer)
	}

	pub fn serialize<S: Serializer>(list: &DoList, serializer: S) -> Result<S::Ok, S::Error> {
		let registry = crate::type_registry();
		let entries = list
			.0
			.iter()
			.map(|item| item.clone_value())
			.collect::<Vec<_>>();
		SceneMapSerializer {
			entries: &entries,
			registry,
		}
		.serialize(serializer)
	}

	pub struct DoListDeserializer<'a> {
		pub registry: &'a TypeRegistry,
	}

	impl<'a, 'de> DeserializeSeed<'de> for DoListDeserializer<'a> {
		type Value = DoList;

		fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
		where
			D: Deserializer<'de>,
		{
			SceneMapDeserializer {
				registry: self.registry,
			}
			.deserialize(deserializer)?
			.into_iter()
			.map(|entry| {
				do_from_reflect(entry, self.registry).map_err(|e| {
					Error::custom(format_args!(
						"failed to downcast {:?} to `Box<dyn Do>`",
						e.get_represented_type_info()
					))
				})
			})
			.collect::<Result<Vec<_>, _>>()
			.map(DoList)
		}
	}
}

pub fn do_from_reflect(
	entry: Box<dyn Reflect>,
	registry: &TypeRegistry,
) -> Result<Box<dyn Do>, Box<dyn Reflect>> {
	let Some(type_info) = entry.get_represented_type_info() else {
		return Err(entry);
	};
	let Some(registration) = registry.get(type_info.type_id()) else {
		return Err(entry);
	};
	let Some(reflect_do) = registration.data::<ReflectDo>() else {
		return Err(entry);
	};

	reflect_do.get_boxed(entry)
}

pub fn do_ref_from_reflect<'a>(
	entry: &'a dyn Reflect,
	registry: &TypeRegistry,
) -> Option<&'a dyn Do> {
	let type_info = entry.get_represented_type_info()?;
	let registration = registry.get(type_info.type_id())?;
	let reflect_do = registration.data::<ReflectDo>()?;

	reflect_do.get(entry)
}

#[derive(TypePath, Default, Deref, DerefMut)]
pub struct DoList(pub Vec<Box<dyn Do>>);

impl Debug for DoList {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.0
			.iter()
			.map(|item| item.as_reflect())
			.collect::<Vec<_>>()
			.fmt(f)
	}
}

impl Clone for DoList {
	fn clone(&self) -> Self {
		Self(self.0.iter().map(|item| item.clone_do()).collect())
	}
}

impl Reflect for DoList {
	fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
		None
	}

	fn into_any(self: Box<Self>) -> Box<dyn Any> {
		self
	}

	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
		self
	}

	fn as_reflect(&self) -> &dyn Reflect {
		self
	}

	fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
		self
	}

	fn apply(&mut self, value: &dyn Reflect) {
		let ReflectRef::List(value) = value.reflect_ref() else {
			error!("tried to apply a non-List value to DoList");
			return;
		};
		let shorter = self.0.len().min(value.len());
		for i in 0..shorter {
			let val = value
				.get(i)
				.expect("value should exist since i < the shortest len")
				.clone_value();
			let ty = val.get_represented_type_info();
			let Ok(val) = do_from_reflect(val, &crate::type_registry().read()) else {
				error!("failed to downcast {ty:?} to `Box<dyn Do>`");
				continue;
			};
			self.0[i] = val;
		}
		if value.len() > self.0.len() {
			for i in shorter..value.len() {
				self.push(
					value
						.get(i)
						.expect("value should exist since i < value.len()")
						.clone_value(),
				);
			}
		}
	}

	fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
		*self = *value.downcast()?;
		Ok(())
	}

	fn reflect_ref(&self) -> ReflectRef {
		ReflectRef::List(self)
	}

	fn reflect_mut(&mut self) -> ReflectMut {
		ReflectMut::List(self)
	}

	fn reflect_owned(self: Box<Self>) -> ReflectOwned {
		ReflectOwned::List(self)
	}

	fn clone_value(&self) -> Box<dyn Reflect> {
		Box::new(self.clone())
	}
}

impl FromReflect for DoList {
	fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
		let ReflectRef::List(list) = reflect.reflect_ref() else {
			return None;
		};
		Some(Self(
			list.iter()
				.flat_map(|item| {
					do_ref_from_reflect(item, &crate::type_registry().read())
						.or_else(|| {
							error!(
								"{} couldn't be downcast to `Box<dyn Do>`",
								item.reflect_type_path()
							);
							None
						})
						.map(|item| item.clone_do())
				})
				.collect(),
		))
	}
}

impl List for DoList {
	fn get(&self, index: usize) -> Option<&dyn Reflect> {
		self.0.get(index).map(|item| item.as_reflect())
	}

	fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
		self.0.get_mut(index).map(|item| item.as_reflect_mut())
	}

	fn insert(&mut self, index: usize, element: Box<dyn Reflect>) {
		do_from_reflect(element, &crate::type_registry().read()).map_or_else(
			|e| {
				error!(
					"couldn't downcast {} to `Box<dyn Do>`",
					e.reflect_type_path()
				)
			},
			|item| self.0.insert(index, item),
		)
	}

	fn remove(&mut self, index: usize) -> Box<dyn Reflect> {
		self.0.remove(index).into_reflect()
	}

	fn len(&self) -> usize {
		self.0.len()
	}

	fn iter(&self) -> ListIter {
		ListIter::new(self)
	}

	fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {
		self.0.into_iter().map(|item| item.into_reflect()).collect()
	}
}

#[derive(Component, Reflect, Debug, Deref, DerefMut, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct SpawnedAt(pub LoopTime);

#[derive(Component, Reflect, Debug, Deref, DerefMut, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct Lifetime(pub LoopTime);
