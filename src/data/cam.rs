use bevy::prelude::*;
use sond_bevy_enum_components::EnumComponent;

#[derive(Component, EnumComponent)]
pub enum CamNode {
	Anchor,
	Gimbal,
}
