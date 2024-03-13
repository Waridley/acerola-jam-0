use crate::data::phys::ColliderShape;
use bevy::prelude::*;
use bevy_xpbd_3d::parry::shape::SharedShape;
use parking_lot::{RwLock, RwLockReadGuard};
use serde::{Deserialize, Serialize};
use sond_bevy_enum_components::EnumComponent;

#[derive(Component, EnumComponent)]
pub enum CamNode {
	Anchor,
	Gimbal,
}

#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct AvoidOccludingPlayer {
	pub if_in_area: Option<ColliderShape>,
	pub area_transform: Transform,
	#[reflect(ignore)]
	#[serde(skip)]
	pub area_shape: RwLock<Option<SharedShape>>,
}

impl AvoidOccludingPlayer {
	pub fn shape(&self) -> RwLockReadGuard<Option<SharedShape>> {
		if let Some(desc) = self.if_in_area.as_ref().cloned() {
			if self.area_shape.read().is_none() {
				*self.area_shape.write() = Some(desc.into());
			}
		}
		self.area_shape.read()
	}
}
