use crate::data::tl::{TimeLoop, Timeline};
use bevy::prelude::*;

pub struct TimeGraphPlugin;

impl Plugin for TimeGraphPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<TimeLoop>()
			.add_systems(PreUpdate, step_loop)
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
	let prev = tloop.curr;
	*tloop.curr += t.delta();
	for (id, tl) in timelines.iter() {
		let path = asrv
			.get_path(id)
			.map_or_else(String::new, |path| format!("{path}: "));
		if let Some(branch_from) = tl.branch_from.as_ref() {
			if (prev..tloop.curr).contains(&branch_from.1) {
				debug!(target: "time_graph", "{path}: branching from {branch_from:?}")
			}
		}
		for (lt, mom) in tl.moments.range(prev..=tloop.curr) {
			debug!(target: "time_graph", desc = mom.desc, "{path}{}@{lt}", mom.label.as_deref().unwrap_or(""));
			for happen in mom.happenings.iter() {
				trace!(target: "time_graph", "{happen:?}");
				happen.apply(cmds.reborrow());
			}
		}
		if let Some(merge_into) = tl.merge_into.as_ref() {
			if (prev..tloop.curr).contains(&merge_into.1) {
				debug!(target: "time_graph", "{path}: merging into {merge_into:?}")
			}
		}
	}
}

pub fn print_timelines(
	mut events: EventReader<AssetEvent<Timeline>>,
	timelines: Res<Assets<Timeline>>,
) {
	for ev in events.read() {
		match ev {
			AssetEvent::Added { id }
			| AssetEvent::Modified { id }
			| AssetEvent::Removed { id }
			| AssetEvent::Unused { id }
			| AssetEvent::LoadedWithDependencies { id } => {
				let tl = timelines.get(*id).unwrap();
				debug!("{ev:?}: {tl:?}");
			}
		}
	}
}
