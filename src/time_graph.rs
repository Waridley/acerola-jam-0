use crate::{
	data::{
		tl::{Lifetime, LoopTime, PortalTo, SpawnedAt, TimeLoop, Timeline, Trigger, TriggerKind},
		ui::{InteractSign, InteractText},
		Str,
	},
	happens::reset_world,
	player::{player_entity::Root, Action},
	GameState,
};
use bevy::{prelude::*, utils::intern::Interned};
use bevy_xpbd_3d::prelude::CollidingEntities;
use leafwing_input_manager::prelude::ActionState;
use sond_bevy_enum_components::WithVariant;
use std::{cmp::Ordering, f32::consts::TAU, ops::Range};

pub struct TimeGraphPlugin;

impl Plugin for TimeGraphPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(First, handle_lifetimes)
			.add_systems(
				PreUpdate,
				(step_loop, take_portal).run_if(in_state(GameState::Running)),
			)
			.add_systems(Update, seek.run_if(in_state(GameState::ResettingLoop)))
			.add_systems(
				PostUpdate,
				(
					print_timelines,
					check_triggers.run_if(in_state(GameState::Running)),
				),
			);
	}
}

pub fn step_loop(
	cmds: Commands,
	mut tloop: ResMut<TimeLoop>,
	timelines: Res<Assets<Timeline>>,
	asrv: Res<AssetServer>,
	t: Res<Time>,
) {
	let prev = tloop.curr.1;
	tloop.curr.1 += t.delta();
	let id = tloop.curr.0;
	handle_happenings(
		cmds,
		&asrv,
		&timelines,
		// Would seem to make more sense to only handle events that have
		// happened *since* the last update, but using `Bound::Excluded`
		// for `prev` would potentially result in unexpectedly missed
		// events when resetting the loop to exactly `prev`.
		prev..tloop.curr.1,
		id,
	)
}

pub fn handle_happenings(
	mut cmds: Commands,
	asrv: &AssetServer,
	timelines: &Assets<Timeline>,
	range: Range<LoopTime>,
	tl: AssetId<Timeline>,
) {
	let path = asrv
		.get_path(tl)
		.map_or_else(String::new, |path| format!("{path}: "));
	let Some(tl) = timelines.get(tl) else {
		error!("timeline {path} should exist");
		return;
	};
	if let Some(branch_from) = tl.branch_from.as_ref() {
		if range.start < branch_from.1 {
			let end = if range.end < branch_from.1 {
				range.end
			} else {
				branch_from.1
			};
			handle_happenings(
				cmds.reborrow(),
				asrv,
				timelines,
				range.start..end,
				branch_from.0,
			);
		}
		if range.contains(&branch_from.1) {
			info!(target: "time_graph", "{path}: branching from {branch_from:?}")
		}
	}
	for (lt, mom) in tl.moments.range(range.clone()) {
		if mom.disabled {
			debug!(target: "time_graph", "[disabled] {}@{lt}", mom.label.unwrap_or(Str(Interned(""))));
			continue;
		}
		debug!(target: "time_graph", desc = mom.desc.as_deref(), "{path}{}@{lt}", mom.label.unwrap_or(Str(Interned(""))));
		for (i, happenings) in mom.happenings.iter().enumerate() {
			if happenings.disabled {
				debug!(target: "time_graph", "\t└ [disabled] {}", happenings.label.unwrap_or_else(|| (&*format!("{i}")).into()));
				continue;
			} else {
				debug!(target: "time_graph", "\t└ {}", happenings.label.unwrap_or_else(|| (&*format!("{i}")).into()));
			}
			for happen in &happenings.actions {
				happen.apply(cmds.reborrow());
			}
		}
	}
	if let Some(merge_into) = tl.merge_into.as_ref() {
		if range.contains(&merge_into.1) {
			debug!(target: "time_graph", "{path}: merging into {merge_into:?}")
		}
		if range.end > merge_into.1 {
			let start = if range.start > merge_into.1 {
				range.start
			} else {
				merge_into.1
			};
			handle_happenings(cmds, asrv, timelines, start..range.end, merge_into.0);
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
	mut interact_sign: Query<&mut Visibility, With<InteractSign>>,
	mut interact_text: Query<&mut Text, With<InteractText>>,
) {
	let Ok((colliding, inputs)) = player.get_single() else {
		return;
	};

	let mut vis = interact_sign.single_mut();
	let mut text = interact_text.single_mut();
	let mut interact_msg = None;
	for id in colliding.iter().copied() {
		if let Ok(trigger) = triggers.get(id) {
			if let TriggerKind::Interact { message } = trigger.kind {
				interact_msg = Some(message);
				if !inputs.just_pressed(&Action::Interact) {
					continue;
				}
			}
			for to_do in trigger.causes.iter() {
				to_do.apply(cmds.reborrow());
			}
			if trigger.oneshot {
				cmds.entity(id).despawn();
			}
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

pub fn handle_lifetimes(
	mut cmds: Commands,
	q: Query<(Entity, &SpawnedAt, &Lifetime)>,
	tl: Res<TimeLoop>,
) {
	for (id, timestamp, lifetime) in &q {
		if tl.curr.1 - timestamp.0 >= lifetime.0 {
			cmds.entity(id).despawn_recursive();
		}
	}
}

pub fn seek(
	mut cmds: Commands,
	mut tloop: ResMut<TimeLoop>,
	mut next_state: ResMut<NextState<GameState>>,
	t: Res<Time>,
) {
	let dt = t.delta_seconds();
	let TimeLoop {
		ref mut curr,
		resetting_from: from,
		resetting_to: to,
	} = *tloop;
	let prev = curr.1;
	match to.cmp(&from) {
		Ordering::Less => {
			let from_s = from.secs_f32();
			let to_s = to.secs_f32();
			let range = from_s - to_s;
			let t = 1.0 - ((from_s - prev.secs_f32()) / range);
			let speed = ((t - 0.5) * TAU).cos() * 0.5 + 0.5;
			let dt = dt + (dt * speed * 8.0);
			curr.1 -= LoopTime::from(dt);
			if t < 0.4 && t + (dt / range) >= 0.4 {
				cmds.add(reset_world);
			}
			if curr.1 <= to {
				curr.1 = to;
				next_state.set(GameState::Running);
			}
		}
		Ordering::Greater => {
			let from_s = from.secs_f32();
			let to_s = to.secs_f32();
			let range = to_s - from_s;
			let t = (prev.secs_f32() - from_s) / range;
			let speed = ((t - 0.5) * TAU).cos() * 0.5 + 0.5;
			let dt = dt + (dt * speed * 8.0);
			curr.1 += LoopTime::from(dt);
			if t < 0.4 && t + (dt / range) >= 0.4 {
				cmds.add(reset_world);
			}
			if curr.1 >= to {
				curr.1 = to;
				next_state.set(GameState::Running);
			}
		}
		Ordering::Equal => {
			warn!("to == from");
		}
	}
}
