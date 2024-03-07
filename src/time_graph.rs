use crate::{
	data::{
		tl::{PortalTo, TimeLoop, Timeline, Trigger, TriggerKind},
		Str,
	},
	player::{player_entity::Root, Action},
};
use bevy::{prelude::*, utils::intern::Interned};
use bevy_xpbd_3d::prelude::CollidingEntities;
use leafwing_input_manager::prelude::ActionState;
use sond_bevy_enum_components::WithVariant;
use crate::data::ui::InteractSign;

pub struct TimeGraphPlugin;

impl Plugin for TimeGraphPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PreUpdate, (step_loop, take_portal, check_triggers))
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

pub fn check_triggers(
	mut cmds: Commands,
	player: Query<(&CollidingEntities, &ActionState<Action>), WithVariant<Root>>,
	triggers: Query<&Trigger>,
	mut interact_sign: Query<(&mut Text, &mut Visibility), With<InteractSign>>,
) {
	let Ok((colliding, inputs)) = player.get_single() else {
		return;
	};
	
	let (mut text, mut vis) = interact_sign.single_mut();
	let mut interact_msg = None;
	for id in colliding.iter().copied() {
		if let Ok(trigger) = triggers.get(id) {
			if let TriggerKind::Interact { message } = trigger.kind {
				interact_msg = Some(message);
				if !inputs.pressed(&Action::Interact) {
					continue;
				}
			}
			for to_do in trigger.causes.iter() {
				to_do.apply(cmds.reborrow());
			}
			cmds.entity(id).despawn();
		}
	}
	if let Some(msg) = interact_msg {
		if *vis != Visibility::Visible {
			*vis = Visibility::Visible;
		}
		if *text.sections[0].value != **msg {
			text.sections[0].value = msg.to_string();
		}
	} else if *vis == Visibility::Visible {
		*vis = Visibility::Hidden
	}
}
