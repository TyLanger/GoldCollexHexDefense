use bevy::prelude::*;

use crate::{gold::*, hexes::*, palette::*};

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TowerBuiltEvent>()
            .add_event::<PlaceTowerPreviewEvent>()
            //.add_system(spawn_tower)
            //.add_system(tower_input)
            .add_system(tower_mouse_input)
            .add_system(spawn_tower_preview)
            .add_system(preview_paid_for)
            .add_system(remove_tower);
        //.add_system(rotate_sprite);
    }
}

#[derive(Component)]
pub struct Tower {
    pub coords: HexCoords,
    pub refund: u32,
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
                                color: LIGHT_BLUE,
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
    mut commands: Commands,
    mut ev_pile_cap: EventReader<PileCapEvent>,
    q_preview_towers: Query<(Entity, &Children, &Hex, &GoldPile), With<TowerPreview>>,
    mut q_child: Query<&mut Sprite>,
) {
    for ev in ev_pile_cap.iter() {
        for (ent, children, hex, pile) in q_preview_towers.iter() {
            if ev.coords.is_same(hex.coords) {
                //println!("Upgrade {:?}", hex.coords);

                // change the color of the preview to a tower color
                for &child in children.iter() {
                    let sprite = q_child.get_mut(child);
                    match sprite {
                        Ok(mut s) => {
                            s.color = DARK_BLUE;
                        }
                        Err(e) => {
                            error!("Error getting child sprite: {e}");
                        }
                    }
                }

                commands
                    .entity(ent)
                    //.remove_children(children)
                    .remove_bundle::<PreviewTowerBundle>()
                    .insert(Tower {
                        coords: ev.coords,
                        refund: (pile.gold_cap as f32 * 0.8) as u32,
                    })
                    .insert(GoldSpawner::new());

                break;
            }
        }
    }
}

fn remove_tower(
    mut commands: Commands,
    mut ev_remove: EventReader<PileRemoveEvent>,
    mut ev_spawn_gold: EventWriter<SpawnGoldEvent>,
    q_towers: Query<(
        Entity,
        &Children,
        &Transform,
        &Hex,
        Option<&GoldPile>,
        Option<&TowerPreview>,
        Option<&Tower>,
    )>,
    //mut q_child: Query<&mut Sprite>,
) {
    for ev in ev_remove.iter() {
        for (ent, children, trans, hex, _opt_pile, opt_preview, opt_tower) in q_towers.iter() {
            if ev.coords.is_same(hex.coords) {
                let mut opt_count = 0;
                if let Some(_) = opt_preview {
                    opt_count += 1;
                }
                if let Some(_) = opt_tower {
                    opt_count += 1;
                }
                if opt_count == 0 {
                    println!("No optionals");
                    break;
                }

                let mut pile_count = 0;

                // if let Some(pile) = opt_pile {
                //     pile_count = pile.count;

                // } else
                if let Some(tower) = opt_tower {
                    pile_count = tower.refund;
                }
                //println!("Pile count: {:?}", pile_count);

                for _ in 0..pile_count {
                    ev_spawn_gold.send(SpawnGoldEvent {
                        position: trans.translation,
                    });
                }

                for &child in children {
                    //println!("despawning children");
                    // runs once
                    commands.entity(child).despawn_recursive();
                }

                match opt_preview {
                    Some(_) => {
                        commands.entity(ent).remove::<TowerPreview>();
                    }
                    None => {
                        //println!("No Preview");
                    }
                }

                match opt_tower {
                    Some(_) => {
                        commands.entity(ent).remove::<GoldSpawner>();
                        commands.entity(ent).remove::<Tower>();
                        // remove surrounding spawners
                    }
                    None => {
                        //println!("No Tower");
                    }
                }
            }
        }
    }
}
