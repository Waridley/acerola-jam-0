use crate::{
	data::{
		tl::{PortalTo, TimeLoop, Timeline},
		Str,
	},
	player::player_entity::Root,
};
use bevy::{prelude::*, utils::intern::Interned};
use bevy_xpbd_3d::prelude::CollidingEntities;
use sond_bevy_enum_components::WithVariant;

pub struct TimeGraphPlugin;

impl Plugin for TimeGraphPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PreUpdate, (step_loop, take_portal))
			.add_systems(Update, print_timelines);
	}
}

pub fn step_loop(
	mut cmds: Commands,
	mut tloop: ResMut<TimeLoop>,
	timelines: Res<Assets<Timeline>>,
	asrv: Res<AssetServer>,
	t: Res<Time>,
) {
	let prev = tloop.curr.1;
	*tloop.curr.1 += t.delta();
	for (id, tl) in timelines.iter() {
		let path = asrv
			.get_path(id)
			.map_or_else(String::new, |path| format!("{path}: "));
		if let Some(branch_from) = tl.branch_from.as_ref() {
			if (prev..tloop.curr.1).contains(&branch_from.1) {
				info!(target: "time_graph", "{path}: branching from {branch_from:?}")
			}
		}
		for (lt, mom) in tl.moments.range(prev..=tloop.curr.1) {
			if mom.disabled {
				debug!(target: "time_graph", "[disabled] {}@{lt}", mom.label.unwrap_or(Str(Interned(""))));
				continue;
			}
			debug!(target: "time_graph", desc = mom.desc.as_deref(), "{path}{}@{lt}", mom.label.unwrap_or(Str(Interned(""))));
			for happenings in mom.happenings.iter() {
				if happenings.disabled {
					debug!(target: "time_graph", "[disabled] {}", happenings.label.unwrap_or(Str(Interned(""))));
					continue;
				}
				if let Some(label) = happenings.label.as_ref() {
					debug!(target: "time_graph", "{label}");
				}
				for happen in &happenings.actions {
					happen.apply(cmds.reborrow());
				}
			}
		}
		if let Some(merge_into) = tl.merge_into.as_ref() {
			if (prev..tloop.curr.1).contains(&merge_into.1) {
				debug!(target: "time_graph", "{path}: merging into {merge_into:?}")
			}
		}
	}
}

pub fn print_timelines(mut events: EventReader<AssetEvent<Timeline>>, assets: Res<AssetServer>) {
	for ev in events.read() {
		match ev {
			AssetEvent::Added { id }
			| AssetEvent::Modified { id }
			| AssetEvent::Removed { id }
			| AssetEvent::Unused { id }
			| AssetEvent::LoadedWithDependencies { id } => {
				let path = assets.get_path(*id).map(|path| path.to_string());
				let path = path.as_deref().unwrap_or("");
				debug!("{path}: {ev:?}");
			}
		}
	}
}

pub fn take_portal(
	player: Query<&CollidingEntities, WithVariant<Root>>,
	portals: Query<&PortalTo>,
	mut tl: ResMut<TimeLoop>,
) {
	let Ok(colliding) = player.get_single() else {
		return;
	};
	for id in colliding.iter().copied() {
		if let Ok(portal) = portals.get(id) {
			tl.curr = portal.0;
		}
	}
}
