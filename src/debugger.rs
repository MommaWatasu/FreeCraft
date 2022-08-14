use bevy::{
    prelude::*,
    text::TextStyle,
    ui::entity::TextBundle
};

use crate::player::Player;

#[derive(Default)]
pub struct Debugger {
    enable: bool,
    entity: Option<Entity>
}

fn setup_debugger(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) -> Entity {
    commands.spawn_bundle(TextBundle::from_section(
            "XYZ: 0.0 / 0.0 / 0.0",
            TextStyle {
                font: asset_server.load("FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::WHITE,
            }
        )
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        })
    ).id()
}

pub fn update_debugger(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut debugger: ResMut<Debugger>,
    keyboard_input: Res<Input<KeyCode>>,
    player_entity: Query<&Transform, With<Player>>,
    mut query: Query<&mut Text>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        if debugger.enable {
            debugger.enable = false;
            commands.entity(debugger.entity.unwrap()).despawn();
        } else {
            debugger.enable = true;
            debugger.entity = Some(setup_debugger(commands, asset_server));
        }
    }
    if debugger.enable {
        let translation
            = match player_entity.get_single() {
            Ok(transform) => transform.translation,
            _ => {
                error!("Player not found.");
                return;
            }
        };
        for mut text in &mut query {
            text.sections[0].value = format!("XYZ: {} / {} / {}", translation.x, translation.y, translation.z)
        }
    }
}