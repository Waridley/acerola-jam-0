use crate::{asset_server, scn::spawn_environment};
use bevy::{
	asset::AssetPath,
	ecs::system::SystemId,
	prelude::{Deref, TypePath, *},
	reflect::{
		DynamicTuple, DynamicTupleStruct, GetTypeRegistration, ReflectFromPtr, ReflectMut,
		ReflectOwned, ReflectRef, TupleStructFieldIter, TypeInfo, TypeRegistration, Typed,
	},
	render::render_resource::Face,
	utils::{
		intern::{Interned, Interner},
		HashMap,
	},
};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use std::{
	any::Any,
	fmt,
	fmt::{Display, Formatter},
	marker::PhantomData,
};

pub mod cam;
pub mod phys;
pub mod tl;
pub mod ui;

pub struct DataPlugin;

impl Plugin for DataPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<LoadAsset<Image>>()
			.register_type::<LoadSprite3d>()
			.add_systems(
				Last,
				(
					replace_paths_with_handles::<Image>,
					replace_sprite3ds_with_handles,
					set_atlas_3d_meshes,
				),
			)
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

impl From<Str> for String {
	fn from(value: Str) -> Self {
		value.to_string()
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
#[reflect(Component, Serialize, Deserialize)]
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

#[derive(Component, Reflect, Clone, Debug, Deref, DerefMut, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize, where T: Serialize, T: DeserializeOwned)]
pub struct InlineAsset<T: Asset + Reflect + FromReflect> {
	#[deref]
	pub value: T,
}

impl<T: Asset + Serialize + DeserializeOwned + Reflect + FromReflect> InlineAsset<T> {
	pub fn into_handle(self) -> Handle<T> {
		asset_server().add(self.value)
	}
}

#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
#[serde(default)]
pub struct LoadSprite3d {
	pub size: Vec2,
	pub atlas_layout: Option<LoadAtlas3d>,
	pub transform: Transform,
	pub material: LoadStdMat,
}

#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct LoadAtlas3d {
	pub tile_size: Vec2,
	pub columns: usize,
	pub rows: usize,
	#[serde(default)]
	pub padding: Option<Vec2>,
	#[serde(default)]
	pub offset: Option<Vec2>,
}

impl Default for LoadSprite3d {
	fn default() -> Self {
		Self {
			size: Vec2::ONE,
			atlas_layout: None,
			transform: default(),
			material: LoadStdMat {
				base_color_texture: Some("bevy_logo_dark.png".into()),
				base_color: Color::WHITE,
				alpha_mode: LoadAlphaMode::Mask(0.5),
				double_sided: true,
				emissive: Color::BLACK,
				..default()
			},
		}
	}
}

pub fn replace_sprite3ds_with_handles(
	mut cmds: Commands,
	q: Query<(Entity, &LoadSprite3d)>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
	srv: Res<AssetServer>,
) {
	for (id, to_load) in &q {
		let mut cmds = cmds.entity(id);
		let LoadSprite3d {
			size,
			atlas_layout,
			transform,
			material,
		} = to_load.clone();

		let material = mats.add(material.load_using(&srv));

		let mesh = if let Some(LoadAtlas3d {
			tile_size,
			columns,
			rows,
			padding,
			offset,
		}) = atlas_layout
		{
			let layout = TextureAtlasLayout::from_grid(tile_size, columns, rows, padding, offset);
			let template = Rectangle::new(size.x, size.y).mesh();
			let size = layout.size;
			let len = layout.textures.len();
			let mut atlas_meshes = Vec::with_capacity(len);
			for rect in &layout.textures {
				let u0 = rect.min.x / size.x;
				let u1 = rect.max.x / size.x;
				let v0 = rect.min.y / size.y;
				let v1 = rect.max.y / size.y;

				let mesh = template.clone().with_inserted_attribute(
					Mesh::ATTRIBUTE_UV_0,
					vec![[u1, v0], [u0, v0], [u0, v1], [u1, v1]],
				);
				atlas_meshes.push(meshes.add(mesh));
			}
			let init_mesh = atlas_meshes[0].clone();
			cmds.insert((
				SpriteSheet3dMeshes(atlas_meshes),
				TextureAtlas {
					layout: atlas_layouts.add(layout),
					index: 0,
				},
			));

			init_mesh
		} else {
			meshes.add(Rectangle::new(size.x, size.y))
		};

		let pbr = PbrBundle {
			mesh,
			material,
			transform,
			..default()
		};

		cmds.insert((pbr,));
		cmds.remove::<LoadSprite3d>();
	}
}

#[derive(Component, Debug, Deref, DerefMut)]
pub struct SpriteSheet3dMeshes(pub Vec<Handle<Mesh>>);

pub fn set_atlas_3d_meshes(
	mut q: Query<(&mut Handle<Mesh>, &SpriteSheet3dMeshes, &TextureAtlas), Changed<TextureAtlas>>,
) {
	for (mut handle, meshes, atlas) in &mut q {
		let new = meshes[atlas.index].clone();
		if *handle != new {
			*handle = new
		}
	}
}

#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
#[serde(default)]
pub struct LoadStdMat {
	pub base_color_texture: Option<AssetPath<'static>>,
	pub base_color: Color,
	pub alpha_mode: LoadAlphaMode,
	pub unlit: bool,
	pub double_sided: bool,
	pub emissive: Color,
	pub emissive_texture: Option<AssetPath<'static>>,
	pub perceptual_roughness: f32,
	pub reflectance: f32,
	pub cull_mode: Option<CullFace>,
}

#[derive(Reflect, Default, Copy, Clone, Debug, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum CullFace {
	Front,
	#[default]
	Back,
}

impl From<CullFace> for Face {
	fn from(value: CullFace) -> Self {
		match value {
			CullFace::Front => Face::Front,
			CullFace::Back => Face::Back,
		}
	}
}

impl Default for LoadStdMat {
	fn default() -> Self {
		Self {
			base_color_texture: None,
			base_color: Color::WHITE,
			alpha_mode: default(),
			unlit: false,
			double_sided: true,
			emissive: Color::BLACK,
			emissive_texture: None,
			perceptual_roughness: 0.5,
			reflectance: 0.5,
			cull_mode: Some(CullFace::Back),
		}
	}
}

impl LoadStdMat {
	pub fn load_using(self, server: &AssetServer) -> StandardMaterial {
		let Self {
			base_color_texture,
			base_color,
			alpha_mode,
			unlit,
			double_sided,
			emissive,
			emissive_texture,
			perceptual_roughness,
			reflectance,
			cull_mode,
		} = self;

		StandardMaterial {
			base_color_texture: base_color_texture.map(|path| server.load(path)),
			base_color,
			alpha_mode: alpha_mode.into(),
			// depth_bias,
			// depth_map,
			// parallax_depth_scale,
			// parallax_mapping_method,
			// max_parallax_layer_count,
			// lightmap_exposure,
			// opaque_render_method,
			unlit,
			double_sided,
			emissive,
			emissive_texture: emissive_texture.map(|path| server.load(path)),
			perceptual_roughness,
			// metallic,
			// metallic_roughness_texture,
			reflectance,
			// diffuse_transmission,
			// specular_transmission,
			// thickness,
			// ior,
			// attenuation_distance,
			// attenuation_color,
			// normal_map_texture,
			// flip_normal_map_y,
			// occlusion_texture,
			cull_mode: cull_mode.map(Into::into),
			// fog_enabled,
			// deferred_lighting_pass_id,
			..default()
		}
	}
}

#[derive(Reflect, Clone, Copy, Debug, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum LoadAlphaMode {
	Opaque,
	Mask(f32),
	Blend,
	Premultiplied,
	Add,
	Multiply,
}

impl Default for LoadAlphaMode {
	fn default() -> Self {
		Self::Mask(0.5)
	}
}

impl From<LoadAlphaMode> for AlphaMode {
	fn from(value: LoadAlphaMode) -> Self {
		match value {
			LoadAlphaMode::Opaque => AlphaMode::Opaque,
			LoadAlphaMode::Mask(m) => AlphaMode::Mask(m),
			LoadAlphaMode::Blend => AlphaMode::Blend,
			LoadAlphaMode::Premultiplied => AlphaMode::Premultiplied,
			LoadAlphaMode::Add => AlphaMode::Add,
			LoadAlphaMode::Multiply => AlphaMode::Multiply,
		}
	}
}
