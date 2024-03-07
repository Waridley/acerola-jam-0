use bevy::prelude::*;
use crate::data::ui::{default_interact_msg, InteractSign};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup);
	}
}

pub fn setup(
	mut cmds: Commands,
) {
	cmds.spawn((
		TextBundle {
			text: Text::from_section(default_interact_msg(), TextStyle::default()),
			visibility: Visibility::Hidden,
			..default()
		},
		Label,
		InteractSign,
	));
}