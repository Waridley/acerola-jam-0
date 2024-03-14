use crate::data::Str;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct InteractSign;

#[derive(Component, Copy, Clone, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct InteractText;

pub fn default_interact_msg() -> Str {
	"Interact".into()
}

#[derive(Resource, Clone, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct InteractIcon(pub Handle<Image>);
