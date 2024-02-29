use std::ops::Index;
use std::time::Duration;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Resource, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct Timelines(pub Vec<Timeline>);

impl Timelines {
	pub fn get(&self, t: T) -> Option<&Moment> {
		self.0.get(t.timeline)
			.and_then(|tl| tl.get(&t.time))
	}
	
	pub fn get_mut(&mut self, t: T) -> Option<&mut Moment> {
		self.0.get_mut(t.timeline)
			.and_then(|tl| tl.get_mut(&t.time))
	}
}

#[derive(Debug, Default, Deref, DerefMut, Reflect)]
pub struct Timeline {
	#[deref]
	pub moments: HashMap<Duration, Moment>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Reflect)]
pub struct Moment {

}

pub struct T {
	pub timeline: usize,
	pub time: Duration,
}

impl Index<T> for Timelines {
	type Output = Moment;
	
	fn index(&self, index: T) -> &Self::Output {
		&self.0[index.timeline].moments[&index.time]
	}
}
