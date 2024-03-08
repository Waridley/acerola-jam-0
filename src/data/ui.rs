use crate::data::Str;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Default, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct InteractSign;

pub fn default_interact_msg() -> Str {
	"Interact".into()
}
