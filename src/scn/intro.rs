use crate::{
	data::tl::{TimeLoop, Timeline},
	player::player_entity::Root,
};
use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{Collider, CollidingEntities, Sensor};
use serde::{Deserialize, Serialize};
use sond_bevy_enum_components::WithVariant;

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<PreventPortalDisable>()
			.add_systems(Startup, setup)
			.add_systems(PreUpdate, prevent_portal_disable);
	}
}

pub fn setup(mut cmds: Commands) {
	cmds.spawn((
		Collider::sphere(0.5),
		Sensor,
		TransformBundle::from_transform(Transform::from_translation(Vec3::NEG_X * 4.0)),
		PreventPortalDisable,
	));
}

#[derive(Component, Reflect, Copy, Clone, Default, Debug, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct PreventPortalDisable;

pub fn prevent_portal_disable(
	mut cmds: Commands,
	player: Query<&CollidingEntities, WithVariant<Root>>,
	prevention_button: Query<Entity, With<PreventPortalDisable>>,
	mut timelines: ResMut<Assets<Timeline>>,
	tloop: Res<TimeLoop>,
) {
	let Ok(player) = player.get_single() else {
		return;
	};

	for id in player.iter().copied() {
		if let Ok(id) = prevention_button.get(id) {
			let tl = tloop.curr.0;
			let Some(tl) = timelines.get_mut(tl) else {
				error!("No timeline for {tl:?}");
				return;
			};
			let Some(mom) = tl
				.moments
				.values_mut()
				.find(|mom| mom.label == Some("despawn_orb".into()))
			else {
				error!("No moment named 'despawn_orb'");
				return;
			};
			mom.disabled = true;
			info!("Disabled portal despawner");
			cmds.entity(id).despawn();
		}
	}
}
