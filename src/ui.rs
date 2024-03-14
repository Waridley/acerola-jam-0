use crate::data::ui::{default_interact_msg, InteractIcon, InteractSign, InteractText};
use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup);
	}

	fn finish(&self, app: &mut App) {
		let srv = app.world.resource::<AssetServer>();
		let icon = srv.load("ui/keys/KeyE.png");
		app.insert_resource(InteractIcon(icon));
	}
}

pub fn setup(mut cmds: Commands, interact_icon: Res<InteractIcon>) {
	cmds.spawn((
		NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Row,
				justify_self: JustifySelf::Center,
				top: Val::Px(300.0),
				..default()
			},
			visibility: Visibility::Hidden,
			..default()
		},
		InteractSign,
	))
	.with_children(|cmds| {
		cmds.spawn(ImageBundle {
			image: UiImage::new(interact_icon.0.clone()),
			style: Style {
				margin: UiRect::all(Val::Px(16.0)),
				min_width: Val::Px(32.0),
				..default()
			},
			..default()
		});
		cmds.spawn((
			TextBundle {
				text: Text::from_section(
					default_interact_msg(),
					TextStyle {
						font_size: 48.0,
						..default()
					},
				),
				style: Style {
					align_self: AlignSelf::Center,
					..default()
				},
				..default()
			},
			Label,
			InteractText,
		));
	});
}
