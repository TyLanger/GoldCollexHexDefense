use bevy::prelude::*;

use crate::hexes::Selection;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_tower);
    }
}

#[derive(Component)]
struct Tower;

struct SpawnTowerEvent;

fn spawn_tower(
    mut commands: Commands,
    q_selection: Query<&Transform, With<Selection>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for trans in q_selection.iter() {
            // Rectangle
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.25, 0.25, 0.75),
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    transform: Transform {
                        translation: trans.translation,
                        ..default()
                    },
                    ..default()
                })
                .insert(Tower);
        }
    }
}
