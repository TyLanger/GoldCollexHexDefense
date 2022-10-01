use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::utils::Duration;

use crate::hexes::{Hex, HexCoords};
use crate::tower::{Tower, TowerBuiltEvent};
use crate::MouseWorldPos;

pub struct GoldPlugin;

impl Plugin for GoldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ModifySpawnerEvent>()
            .add_event::<PileCapEvent>()
            .add_startup_system(spawn_pile)
            .add_system(spawn_gold)
            .add_system(place_spawner)
            .add_system(remove_spawner)
            .add_system(move_gold)
            .add_system(check_mouse)
            .add_system(store_gold);
    }
}

#[derive(Component)]
pub struct GoldSpawner {
    timer: Timer,
    gold_gen: u32,
}

impl GoldSpawner {
    pub fn new() -> Self {
        GoldSpawner {
            timer: Timer::new(Duration::from_secs_f32(3.0), true),
            gold_gen: 1,
        }
    }
}

pub struct ModifySpawnerEvent {
    pub coords: HexCoords,
    pub modification: Modification,
}

pub enum Modification {
    Remove,
    Hide,
    Upgrade,
}

#[derive(Component)]
struct Gold;

#[derive(Component)]
struct MouseFollow;

// need to move mouse close to pick up gold
// but then need to move farther away to break the tether and drop it
const TETHER_BREAK_DIST: f32 = 250.0;
const TETHER_ENTER_DIST: f32 = 90.0;
const GOLD_MOVE_SPEED: f32 = 225.0;

#[derive(Component)]
pub struct GoldPile {
    count: u32,
    gold_cap: u32,
}

impl GoldPile {
    pub fn new(cap: u32) -> Self {
        GoldPile {
            count: 0,
            gold_cap: cap,
        }
    }
}

pub struct PileCapEvent {
    pub coords: HexCoords,
}

fn store_gold(
    mut commands: Commands,
    q_gold: Query<(Entity, &Transform, &Gold)>,
    mut q_pile: Query<(&Transform, &mut GoldPile, &Hex)>,
    mut ev_cap: EventWriter<PileCapEvent>,
) {
    for (gold_ent, gold_trans, gold) in q_gold.iter() {
        for (pile_trans, mut pile, hex) in q_pile.iter_mut() {
            if let Some(_) = collide(
                gold_trans.translation,
                Vec2::new(8., 12.),
                pile_trans.translation,
                Vec2::new(20., 20.),
            ) {
                if pile.count < pile.gold_cap {
                    pile.count += 1;
                    //println!("Plink! {:?}", pile.count);
                    commands.entity(gold_ent).despawn_recursive();
                    if pile.count == pile.gold_cap {
                        //println!("Cap reached!");
                        ev_cap.send(PileCapEvent { coords: hex.coords });
                    }
                }
            }
        }
    }
}

fn spawn_pile(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::ANTIQUE_WHITE,
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.3,
            }),
            ..default()
        })
        .insert(GoldPile::new(500));
}

fn spawn_gold(
    mut commands: Commands,
    mut q_gold_spawners: Query<(&Transform, &mut GoldSpawner)>,
    time: Res<Time>,
) {
    for (trans, mut spawner) in q_gold_spawners.iter_mut() {
        if spawner.timer.tick(time.delta()).just_finished() {
            //println!("spawn a gold");
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::GOLD,
                        custom_size: Some(Vec2::new(8.0, 12.)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3 {
                        x: trans.translation.x,
                        y: trans.translation.y,
                        z: 0.3,
                    }),
                    ..default()
                })
                .insert(Gold);
        }
    }
}

fn remove_spawner(
    mut commands: Commands,
    mut ev_tower_built: EventReader<TowerBuiltEvent>,
    q_spawners: Query<(Entity, &Hex), With<GoldSpawner>>,
) {
    for ev in ev_tower_built.iter() {
        for (ent, hex) in q_spawners.iter() {
            if ev.coords.is_same(hex.coords) {
                commands.entity(ent).remove::<GoldSpawner>();
                // no other hexes will have the same coords.
                // move on to the next event
                break;
            }
        }
    }
}

fn place_spawner(
    mut commands: Commands,
    mut ev_tower_built: EventReader<TowerBuiltEvent>,
    q_hexes: Query<(Entity, &Hex), Without<GoldSpawner>>,
    q_tower: Query<&Tower>,
) {
    for ev in ev_tower_built.iter() {
        let neighbours = ev.coords.get_neighbours();

        for (ent, hex) in q_hexes.iter() {
            for n in neighbours.iter() {
                let mut tower_there = false;
                for t in q_tower.iter() {
                    // don't spawn if there is already a tower there
                    // towers are on top of a hex, not inserted
                    // so can't do Without<Tower>
                    // But maybe they should be
                    if t.coords.is_same(hex.coords) {
                        tower_there = true;
                    }
                }
                if n.is_same(hex.coords) && !tower_there {
                    //println!("Placed a gold spawner");
                    commands.entity(ent).insert(GoldSpawner::new());
                }
            }
        }
    }
}

fn check_mouse(
    mut commands: Commands,
    q_gold: Query<(Entity, &Transform, Option<&MouseFollow>), With<Gold>>,
    mouse: Res<MouseWorldPos>,
) {
    for (gold_ent, gold_trans, gold_follow) in q_gold.iter() {
        match gold_follow {
            Some(_) => {
                // following the mouse
                if Vec2::distance(gold_trans.translation.truncate(), mouse.0) > TETHER_BREAK_DIST {
                    commands.get_or_spawn(gold_ent).remove::<MouseFollow>();
                }
            }
            _ => {
                // not following

                if Vec2::distance(gold_trans.translation.truncate(), mouse.0) < TETHER_ENTER_DIST {
                    commands.get_or_spawn(gold_ent).insert(MouseFollow);
                }
            }
        }
    }
}

fn move_gold(
    mut q_gold: Query<&mut Transform, With<MouseFollow>>,
    mouse: Res<MouseWorldPos>,
    time: Res<Time>,
) {
    for mut gold in q_gold.iter_mut() {
        let dir = mouse.0 - gold.translation.truncate();
        gold.translation +=
            dir.normalize_or_zero().extend(0.0) * GOLD_MOVE_SPEED * time.delta_seconds();
    }
}
