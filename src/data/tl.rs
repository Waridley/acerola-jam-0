use bevy::{
	asset::{io::Reader, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext},
	prelude::*,
	reflect::{TypeRegistry, TypeRegistryArc},
	scene::{serde::SceneMapDeserializer, SceneLoaderError},
};
use humantime::DurationError;
use serde::{
	de::{DeserializeSeed, Error, MapAccess, Visitor},
	Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
	borrow::Cow,
	collections::BTreeMap,
	fmt::{Debug, Display, Formatter},
	ops::{Index, IndexMut},
	str::FromStr,
	time::Duration,
};

pub struct TimeDataPlugin;

impl Plugin for TimeDataPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<TimeLoop>()
			.init_asset::<Timeline>()
			.register_type::<Print>();
	}

	fn finish(&self, app: &mut App) {
		let registry = app.world.resource::<AppTypeRegistry>();
		app.register_asset_loader(TimelineLoader {
			registry: registry.0.clone(),
		});

		let assets = app.world.resource::<AssetServer>();

		let tl = assets.load("tl/main_timeline.tl.ron");
		app.insert_resource(MainTimeline(tl));
	}
}

/// Seconds since the beginning of the loop
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

#[derive(Asset, TypePath, Debug, Default, Deref, DerefMut)]
pub struct Timeline {
	#[deref]
	pub moments: BTreeMap<LoopTime, Moment>,
}

#[derive(Debug, Default)]
pub struct Moment {
	pub label: Option<String>,
	pub desc: Option<String>,
	pub happenings: Vec<Box<dyn Do>>,
}

pub struct T {
	pub timeline: AssetId<Timeline>,
	pub time: LoopTime,
}

impl Index<T> for Assets<Timeline> {
	type Output = Moment;

	fn index(&self, index: T) -> &Self::Output {
		&self.get(index.timeline).unwrap().moments[&index.time]
	}
}

impl IndexMut<T> for Assets<Timeline> {
	fn index_mut(&mut self, index: T) -> &mut Self::Output {
		self.get_mut(index.timeline)
			.unwrap()
			.moments
			.get_mut(&index.time)
			.unwrap()
	}
}

#[reflect_trait]
pub trait Do: Debug + Send + Sync {
	fn apply(&self, cmds: Commands);
}

/// A dummy happening for debugging time graph code
#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Do, Serialize, Deserialize)]
#[type_path = "happens"]
pub struct Print {
	pub msg: Cow<'static, str>,
}

impl Do for Print {
	fn apply(&self, _cmds: Commands) {
		info!(target: "happens", "{}", &self.msg)
	}
}

pub struct TimelineLoader {
	pub registry: TypeRegistryArc,
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
#[serde(field_identifier, rename_all = "lowercase")]
pub enum TimelineField {
	Moments,
}

pub struct TimelineDeserializer<'de> {
	registry: &'de TypeRegistry,
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
			},
		)
	}
}

pub struct TimelineVisitor<'a> {
	registry: &'a TypeRegistry,
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
		let Some(TimelineField::Moments) = map.next_key()? else {
			return Err(Error::missing_field("moments"));
		};
		let moments = map.next_value_seed(MomentMapDeserializer {
			registry: self.registry,
		})?;

		Ok(Timeline { moments })
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
#[serde(field_identifier, rename_all = "lowercase")]
pub enum MomentField {
	Label,
	Desc,
	Happenings,
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
		
		while let Some(key) = map.next_key()? {
			match key {
				MomentField::Label => label = Some(map.next_value()?),
				MomentField::Desc => desc = Some(map.next_value()?),
				MomentField::Happenings => happenings = Some(map
					.next_value_seed(SceneMapDeserializer {
						registry: self.registry,
					})?
					.into_iter()
					.map(|entry| {
						let type_info = entry.get_represented_type_info().unwrap();
						let registration = self.registry.get(type_info.type_id()).unwrap();
						let reflect_do: &ReflectDo = registration.data::<ReflectDo>().unwrap();
						reflect_do.get_boxed(entry).unwrap()
					})
					.collect()
				),
			}
		}
		Ok(Moment {
			label,
			desc,
			happenings: happenings.unwrap_or_default(),
		})
	}
}

#[derive(Resource, Default, Debug, Reflect)]
pub struct TimeLoop {
	pub curr: LoopTime,
}

#[derive(Resource)]
pub struct MainTimeline(pub Handle<Timeline>);
