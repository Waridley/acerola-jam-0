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
	t: Res<Time>,
) {
	let prev = tloop.curr;
	*tloop.curr += t.delta();
	for (_, tl) in timelines.iter() {
		for (lt, mom) in tl.moments.iter() {
			if prev < *lt && tloop.curr > *lt {
				for happen in mom.happenings.iter() {
					happen.apply(cmds.reborrow());
				}
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
				info!("{ev:?}: {tl:?}");
			}
		}
	}
}
