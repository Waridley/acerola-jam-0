use crate::data::{LoadAlphaMode, LoadStdMat};
use bevy::{
	ecs::system::EntityCommands,
	prelude::*,
	render::{
		mesh::{Indices, PrimitiveTopology},
		render_asset::RenderAssetUsages,
	},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
#[serde(default)]
pub struct LoadSprite3d {
	pub size: Vec2,
	pub atlas_layout: Option<LoadAtlas3d>,
	pub transform: Transform,
	pub material: LoadStdMat,
	pub anchor: Vec2,
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
			anchor: default(),
		}
	}
}

pub struct Sprite3dBundle {
	pub pbr: PbrBundle,
	pub atlas: Option<(SpriteSheet3dMeshes, TextureAtlas)>,
}

impl Sprite3dBundle {
	pub fn insert_self(self, cmds: &mut EntityCommands) {
		let Self { atlas, pbr } = self;
		if let Some(atlas) = atlas {
			cmds.insert(atlas);
		}
		cmds.insert(pbr);
	}
}

impl LoadSprite3d {
	pub fn load_using(
		self,
		srv: &AssetServer,
		meshes: &mut Assets<Mesh>,
		mats: &mut Assets<StandardMaterial>,
		atlas_layouts: &mut Assets<TextureAtlasLayout>,
	) -> Sprite3dBundle {
		let LoadSprite3d {
			size,
			atlas_layout,
			transform,
			material,
			anchor,
		} = self;

		let material = mats.add(material.load_using(srv));

		let (atlas, mesh) = if let Some(atlas_layout) = atlas_layout {
			let (atlas_meshes, atlas) =
				atlas_layout.load_using(size, anchor, meshes, atlas_layouts);
			let init_mesh = atlas_meshes[0].clone();

			(Some((atlas_meshes, atlas)), init_mesh)
		} else {
			(None, meshes.add(mesh_for_sprite(size.x, size.y, anchor)))
		};

		Sprite3dBundle {
			atlas,
			pbr: PbrBundle {
				mesh,
				material,
				transform,
				..default()
			},
		}
	}

	pub fn loader_system(
		this: In<Self>,
		srv: Res<AssetServer>,
		mut meshes: ResMut<Assets<Mesh>>,
		mut mats: ResMut<Assets<StandardMaterial>>,
		mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
	) -> Sprite3dBundle {
		this.0
			.load_using(&srv, &mut meshes, &mut mats, &mut atlas_layouts)
	}
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

impl LoadAtlas3d {
	pub fn load_using(
		self,
		size: Vec2,
		anchor: Vec2,
		meshes: &mut Assets<Mesh>,
		atlas_layouts: &mut Assets<TextureAtlasLayout>,
	) -> (SpriteSheet3dMeshes, TextureAtlas) {
		let Self {
			tile_size,
			columns,
			rows,
			padding,
			offset,
		} = self;

		let layout = TextureAtlasLayout::from_grid(tile_size, columns, rows, padding, offset);
		let template = mesh_for_sprite(size.x, size.y, anchor);
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
		(
			SpriteSheet3dMeshes(atlas_meshes),
			TextureAtlas {
				layout: atlas_layouts.add(layout),
				index: 0,
			},
		)
	}
}

pub fn replace_sprite3ds_with_handles(
	mut cmds: Commands,
	q: Query<(Entity, &LoadSprite3d)>,
	srv: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
	for (id, to_load) in &q {
		let mut cmds = cmds.entity(id);
		let bundle = to_load
			.clone()
			.load_using(&srv, &mut meshes, &mut mats, &mut atlas_layouts);
		bundle.insert_self(&mut cmds);
		cmds.remove::<LoadSprite3d>();
	}
}

#[derive(Component, Debug, Deref, DerefMut)]
pub struct SpriteSheet3dMeshes(pub Vec<Handle<Mesh>>);

/// Like `Rectangle::mesh` but for Z-up basis
pub fn mesh_for_sprite(width: f32, height: f32, anchor: Vec2) -> Mesh {
	let [hw, hh] = [width * 0.5, height * 0.5];
	let Vec2 { x: ax, y: ay } = anchor;
	let positions = vec![
		[hw - ax, 0.0, hh - ay],
		[-(hw + ax), 0.0, hh - ay],
		[-(hw + ax), 0.0, -(hh + ay)],
		[hw - ax, 0.0, -(hh + ay)],
	];
	let normals = vec![[0.0, -1.0, 0.0]; 4];
	let uvs = vec![[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
	let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

	Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::default(),
	)
	.with_inserted_indices(indices)
	.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
	.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
	.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
