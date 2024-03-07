use crate::scn::spawn_environment;
use bevy::{
	asset::AssetPath,
	ecs::system::SystemId,
	prelude::{Deref, TypePath, *},
	reflect::{
		DynamicTuple, DynamicTupleStruct, GetTypeRegistration, ReflectFromPtr, ReflectMut,
		ReflectOwned, ReflectRef, TupleStructFieldIter, TypeInfo, TypeRegistration, Typed,
	},
	utils::{
		intern::{Interned, Interner},
		HashMap,
	},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
	any::Any,
	fmt,
	fmt::{Display, Formatter},
	marker::PhantomData,
};

pub mod cam;
pub mod phys;
pub mod tl;

pub struct DataPlugin;

impl Plugin for DataPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<LoadAsset<Image>>()
			.add_systems(Last, replace_paths_with_handles::<Image>)
			.add_plugins((tl::TimeDataPlugin, phys::PhysDataPlugin));
	}
}

pub static LABEL_CACHE: Interner<str> = Interner::new();

/// An intered `str` for labels.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, TypePath, Deref)]
pub struct Str(pub Interned<str>);

impl FromReflect for Str {
	fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
		if let ReflectRef::TupleStruct(ts) = reflect.reflect_ref() {
			let s = ts.field(0)?;
			let s = s.downcast_ref::<&str>()?;
			Some(Self(LABEL_CACHE.intern(*s)))
		} else {
			None
		}
	}
}

impl TupleStruct for Str {
	fn field(&self, index: usize) -> Option<&dyn Reflect> {
		assert_eq!(index, 0);
		Some(&self.0 .0)
	}

	fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
		assert_eq!(index, 0);
		Some(&mut self.0 .0)
	}

	fn field_len(&self) -> usize {
		1
	}

	fn iter_fields(&self) -> TupleStructFieldIter {
		TupleStructFieldIter::new(self)
	}

	fn clone_dynamic(&self) -> DynamicTupleStruct {
		let mut tuple = DynamicTuple::default();
		tuple.insert(self.0 .0);
		DynamicTupleStruct::from(tuple)
	}
}

impl GetTypeRegistration for Str {
	fn get_type_registration() -> TypeRegistration {
		let mut reg = TypeRegistration::of::<Str>();
		reg.insert::<ReflectFromPtr>(bevy::reflect::FromType::<Self>::from_type());
		reg.insert::<ReflectFromReflect>(bevy::reflect::FromType::<Self>::from_type());
		reg.insert::<ReflectSerialize>(bevy::reflect::FromType::<Self>::from_type());
		reg.insert::<ReflectDeserialize>(bevy::reflect::FromType::<Self>::from_type());
		reg
	}
}

impl Typed for Str {
	fn type_info() -> &'static TypeInfo {
		static CELL: bevy::reflect::utility::NonGenericTypeInfoCell =
			bevy::reflect::utility::NonGenericTypeInfoCell::new();
		CELL.get_or_set(|| {
			let fields = [bevy::reflect::UnnamedField::new::<&'static str>(0)];
			let info = bevy::reflect::TupleStructInfo::new::<Self>(&fields);
			TypeInfo::TupleStruct(info)
		})
	}
}

impl Reflect for Str {
	fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
		Some(<Self as Typed>::type_info())
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
		self.0 .0.apply(value)
	}

	fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
		let ReflectRef::TupleStruct(ts) = value.reflect_ref() else {
			return Err(value);
		};

		let Some(s) = ts.field(0) else {
			return Err(value);
		};
		let Some(s) = s.downcast_ref::<&str>() else {
			return Err(value);
		};
		self.0 = LABEL_CACHE.intern(*s);
		Ok(())
	}

	fn reflect_ref(&self) -> ReflectRef {
		ReflectRef::TupleStruct(self)
	}

	fn reflect_mut(&mut self) -> ReflectMut {
		ReflectMut::TupleStruct(self)
	}

	fn reflect_owned(self: Box<Self>) -> ReflectOwned {
		ReflectOwned::TupleStruct(self)
	}

	fn clone_value(&self) -> Box<dyn Reflect> {
		Box::new(*self)
	}
}

impl From<&str> for Str {
	fn from(value: &str) -> Self {
		Self(LABEL_CACHE.intern(value))
	}
}

impl Display for Str {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str(self.0 .0)
	}
}

impl<'de> Deserialize<'de> for Str {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Ok(<&'de str as Deserialize<'de>>::deserialize(deserializer)?.into())
	}
}

impl Serialize for Str {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(self.0 .0)
	}
}

pub mod entity_path_str {
	use bevy::{animation::EntityPath, core::Name};
	use serde::{Deserialize, Deserializer, Serializer};

	pub fn serialize<S: Serializer>(path: &EntityPath, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = String::new();
		for part in &path.parts {
			s.push_str(part.as_str());
			s.push('.');
		}
		serializer.serialize_str(&s)
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<EntityPath, D::Error> {
		let s = <&'de str as Deserialize<'de>>::deserialize(deserializer)?;
		let parts = s.split('.').map(Name::from).collect();
		Ok(EntityPath { parts })
	}
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct SystemRegistry {
	pub spawn_env: SystemId,
	#[deref]
	pub dynamic: HashMap<Str, SystemId>,
}

impl FromWorld for SystemRegistry {
	fn from_world(world: &mut World) -> Self {
		let spawn_env = world.register_system(spawn_environment);
		Self {
			spawn_env,
			dynamic: default(),
		}
	}
}

#[derive(Component, Reflect, Clone, Debug, Deref, DerefMut, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct LoadAsset<T: Asset>(
	#[deref] pub AssetPath<'static>,
	#[reflect(ignore)]
	#[serde(default)]
	PhantomData<T>,
);

pub fn replace_paths_with_handles<T: Asset>(
	mut cmds: Commands,
	q: Query<(Entity, &LoadAsset<T>)>,
	srv: Res<AssetServer>,
) {
	for (id, LoadAsset(path, _)) in &q {
		let handle = srv.load::<T>(path);
		cmds.entity(id).insert(handle).remove::<LoadAsset<T>>();
	}
}
