use crate::data::phys::ColliderShape;
use bevy::prelude::*;
use bevy_xpbd_3d::parry::shape::SharedShape;
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
	#[reflect(ignore)]
	#[serde(skip)]
	pub area_shape: Option<SharedShape>,
}

impl AvoidOccludingPlayer {
	pub fn shape(&mut self) -> Option<&SharedShape> {
		if self.area_shape.is_none() {
			if let Some(desc) = self.if_in_area.as_ref().cloned() {
				self.area_shape = Some(desc.into());
			}
		}
		self.area_shape.as_ref()
	}
}
