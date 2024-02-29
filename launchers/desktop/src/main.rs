#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

#[allow(unused_imports, clippy::single_component_path_imports)]
#[cfg(all(feature = "bevy_dylib", not(target_arch = "wasm32")))]
use bevy_dylib;

use bevy::prelude::*;
use game_lib::GamePlugin;

fn main() {
	App::new()
		.add_plugins(GamePlugin {
			asset_dir: concat!(env!("PWD"), "/assets/"),
			imported_asset_dir: concat!(env!("PWD"), "/imported_assets/Default"),
		})
		.run()
}
