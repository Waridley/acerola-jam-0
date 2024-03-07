use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::data::Str;

#[derive(Component, Copy, Clone, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct InteractSign;

pub fn default_interact_msg() -> Str {
	"Interact".into()
}
