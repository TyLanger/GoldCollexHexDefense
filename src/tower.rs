use bevy::prelude::*;

use crate::{
    gold::{GoldPile, PileCapEvent},
    hexes::*,
};

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTowerEvent>()
            .add_event::<TowerBuiltEvent>()
            .add_event::<PlaceTowerPreviewEvent>()
            .add_system(spawn_tower)
            .add_system(tower_input)
            .add_system(tower_mouse_input)
            .add_system(spawn_tower_preview)
            .add_system(preview_paid_for);
        //.add_system(rotate_sprite);
    }
}

#[derive(Component)]
pub struct Tower {
    pub coords: HexCoords,
}

#[derive(Component)]
pub struct TowerPreview {}

#[derive(Bundle)]
pub struct PreviewTowerBundle {
    pub preview: TowerPreview,
    pub pile: GoldPile,
}

struct PlaceTowerPreviewEvent {
    position: Vec3,
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

// fn rotate_sprite(
//     mut q_tower: Query<&mut Transform, With<Tower>>,
//     time: Res<Time>,
// ) {
//     for mut tower in q_tower.iter_mut() {
//         // doesn't look very good
//         //tower.rotate_x(time.delta_seconds() * 2.0);
//         // rotating around z looks fine. Kinda looks like a spinning attack charge up

//         // this looks much better
//         let y = (time.seconds_since_startup() * 5.0).sin() as f32;
//         tower.scale = Vec3{x: 1.0, y: y, z: 1.0};
//         // how to look once/twice?
//         // Timer?
//         // add a component, run a timer, remove component?
//     }
// }

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

fn tower_mouse_input(
    mut ev_place_preview: EventWriter<PlaceTowerPreviewEvent>,
    q_selection: Query<(&Transform, &Hex), With<Selection>>,
    input: Res<Input<MouseButton>>,
) {
    if input.just_pressed(MouseButton::Left) {
        for (trans, hex) in q_selection.iter() {
            ev_place_preview.send(PlaceTowerPreviewEvent {
                position: trans.translation,
                coords: hex.coords,
            });
        }
    }
}

// where a tower will be
// Still needs gold brought to it to build it
fn spawn_tower_preview(
    mut commands: Commands,
    mut ev_place_preview: EventReader<PlaceTowerPreviewEvent>,
    q_empty_hexes: Query<(Entity, &Hex), Or<(Without<Tower>, Without<GoldPile>)>>,
) {
    for ev in ev_place_preview.iter() {
        for (ent, hex) in q_empty_hexes.iter() {
            if ev.coords.is_same(hex.coords) {
                // empty hex exists
                commands
                    .entity(ent)
                    .insert_bundle(PreviewTowerBundle {
                        preview: TowerPreview {},
                        pile: GoldPile::new(5),
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(0.35, 0.85, 0.85),
                                custom_size: Some(Vec2::new(20.0, 20.0)),
                                ..default()
                            },
                            transform: Transform {
                                // spawn on top of the underlying hex
                                translation: Vec3 {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.2,
                                },
                                // undo the hex's rotation
                                rotation: Quat::from_rotation_z(-30.0 * DEG_TO_RAD),
                                ..default()
                            },
                            ..default()
                        });
                    });

                // it is now a Hex, TowerPreview, GoldPile,
                // with a sprite child
            }
        }
    }
}

fn preview_paid_for(
    mut ev_pile_cap: EventReader<PileCapEvent>,
    q_preview_towers: Query<(Entity, &Hex), (With<TowerPreview>, With<GoldPile>)>,
) {
    for ev in ev_pile_cap.iter() {
        for (ent, hex) in q_preview_towers.iter() {
            if ev.coords.is_same(hex.coords) {
                println!("Upgrade {:?}", hex.coords);
                break;
            }
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
                        translation: Vec3 {
                            x: ev.position.x,
                            y: ev.position.y,
                            z: 0.2,
                        },
                        ..default()
                    },
                    ..default()
                })
                .insert(Tower { coords: ev.coords });
            //println!("Built a tower");
            ev_tower_built.send(TowerBuiltEvent { coords: ev.coords });
        }
    }
}
