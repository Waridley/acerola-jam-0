use bevy::{
	asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, BoxedFuture, LoadContext},
	ecs::system::Command,
	prelude::*,
	reflect::{serde::TypedReflectDeserializer, TypeRegistry, TypeRegistryArc},
	scene::SceneLoaderError,
};
use humantime::DurationError;
use serde::{
	de::{DeserializeSeed, Error, MapAccess, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
	borrow::{Borrow, Cow},
	collections::BTreeMap,
	fmt::{Debug, Display, Formatter},
	ops::{Index, IndexMut},
	str::FromStr,
	time::Duration,
};

use bevy::utils::CowArc;

use super::Str;
use serde::de::SeqAccess;

pub struct TimeDataPlugin;

impl Plugin for TimeDataPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<TimeLoop>()
			.init_asset::<Timeline>()
			.register_type::<Print>()
			.register_type::<LoopTime>()
			.register_type::<T>()
			.register_type::<TPath>()
			.register_type::<Str>()
			.register_type::<MomentRef>()
			.register_type::<Print>()
			.register_type::<TimeLoop>()
			.register_type::<(AssetPath<'static>, T)>()
			.register_type::<PortalPath>()
			.register_type::<PortalTo>();
	}

	fn finish(&self, app: &mut App) {
		let registry = app.world.resource::<AppTypeRegistry>();
		let asset_server = app.world.resource::<AssetServer>();
		app.register_asset_loader(TimelineLoader {
			registry: registry.0.clone(),
			asset_server: asset_server.clone(),
		});

		let assets = app.world.resource::<AssetServer>();

		let main = assets.load("tl/main.tl.ron");
		let test_branch = assets.load("tl/test_branch.tl.ron");
		app.insert_resource(LoadedTimelines(vec![main, test_branch]));
	}
}

/// Duration since the beginning of the loop
///
/// Serializes in a human-readable format using [humantime]
#[derive(
	Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deref, DerefMut, Reflect,
)]
pub struct LoopTime(Duration);

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
			.map_err(|e| Error::custom(format!("{e}")))
	}
}

impl FromStr for LoopTime {
	type Err = DurationError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		humantime::parse_duration(s).map(Self)
	}
}

impl Display for LoopTime {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", humantime::format_duration(self.0))
	}
}

impl From<f32> for LoopTime {
	fn from(value: f32) -> Self {
		Self(Duration::from_secs_f32(value))
	}
}

impl From<Duration> for LoopTime {
	fn from(value: Duration) -> Self {
		Self(value)
	}
}

impl From<LoopTime> for Duration {
	fn from(value: LoopTime) -> Self {
		value.0
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
		Self("tl/main.tl.ron".into(), default())
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
pub trait Do: Send + Sync {
	fn apply(&self, cmds: Commands);
}

impl<C: Command + Borrow<B>, B: ToOwned<Owned = C> + Send + Sync> Do for B {
	fn apply(&self, mut cmds: Commands) {
		cmds.add(self.to_owned())
	}
}

/// A dummy happening for debugging time graph code
#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct Print {
	pub msg: Cow<'static, str>,
}

impl Command for Print {
	fn apply(self, _world: &mut World) {
		info!(target: "happens::Print", "{}", &self.msg)
	}
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

			let type_info = entry.get_represented_type_info().ok_or_else(|| {
				A::Error::custom(format!("missing represented TypeInfo for {entry:?}"))
			})?;
			let registration = self.registry.get(type_info.type_id()).ok_or_else(|| {
				A::Error::custom(format!("missing TypeRegistration for {entry:?}"))
			})?;
			let reflect_do = registration.data::<ReflectDo>().ok_or_else(|| {
				A::Error::custom(format!("missing `ReflectDo` registration for {entry:?}"))
			})?;
			let action = reflect_do.get_boxed(entry).map_err(|entry| {
				A::Error::custom(format!("failed to get Box<dyn Do> for {entry:?}))"))
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

#[derive(Resource, Default, Debug, Reflect)]
pub struct TimeLoop {
	pub curr: LoopTime,
}

/// Mainly keeps timeline strong handles alive.
#[derive(Resource, Deref, DerefMut)]
pub struct LoadedTimelines(pub Vec<Handle<Timeline>>);

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
