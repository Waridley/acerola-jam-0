use bevy::{
	app::App,
	prelude::{
		Deref, FromReflect, Plugin, Reflect, ReflectDeserialize, ReflectFromReflect,
		ReflectSerialize, TupleStruct, TypePath,
	},
	reflect::{
		DynamicTuple, DynamicTupleStruct, GetTypeRegistration, ReflectFromPtr, ReflectMut,
		ReflectOwned, ReflectRef, TupleStructFieldIter, TypeInfo, TypeRegistration, Typed,
	},
	utils::intern::{Interned, Interner},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
	any::Any,
	fmt,
	fmt::{Display, Formatter},
};

pub mod cam;
pub mod tl;

pub struct DataPlugin;

impl Plugin for DataPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((tl::TimeDataPlugin,));
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
