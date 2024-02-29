use bevy::{app::App, prelude::Plugin};

pub mod tl;

pub struct DataPlugin;

impl Plugin for DataPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((tl::TimeDataPlugin,));
	}
}
