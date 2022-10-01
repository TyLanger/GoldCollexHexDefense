use bevy::prelude::*;

use crate::hexes::*;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnTowerEvent>()
            .add_event::<TowerBuiltEvent>()
            .add_system(spawn_tower)
            .add_system(tower_input);
    }
}

#[derive(Component)]
struct Tower {
    coords: HexCoords,
}

// try to build a tower here
struct SpawnTowerEvent {
    position: Vec3,
    coords: HexCoords,
}

// successfully build
pub struct TowerBuiltEvent {
    pub coords: HexCoords,
}

fn tower_input(
    mut ev_spawn_tower: EventWriter<SpawnTowerEvent>,
    q_selection: Query<(&Transform, &Hex), With<Selection>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for (trans, hex) in q_selection.iter() {
            ev_spawn_tower.send(SpawnTowerEvent {
                position: trans.translation,
                coords: hex.coords,
            });
        }
    }
}

fn spawn_tower(
    mut commands: Commands,
    mut ev_spawn_tower: EventReader<SpawnTowerEvent>,
    mut ev_tower_built: EventWriter<TowerBuiltEvent>,
    q_towers: Query<&Tower>,
) {
    for ev in ev_spawn_tower.iter() {
        let mut can_spawn = true;
        for tower in q_towers.iter() {
            if tower.coords.is_same(ev.coords) {
                can_spawn = false;
                println!("Spawn failed. Tower already here");
                break;
            }
        }

        if can_spawn {
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.25, 0.25, 0.75),
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3 { x: ev.position.x, y: ev.position.y, z: 0.2 },
                        ..default()
                    },
                    ..default()
                })
                .insert(Tower { coords: ev.coords });
            //println!("Built a tower");
            ev_tower_built.send(TowerBuiltEvent {coords: ev.coords});
        }
    }
}
