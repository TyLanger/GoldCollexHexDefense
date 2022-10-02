use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::utils::Duration;

use crate::hexes::{Hex, HexCoords, Selection, DEG_TO_RAD};
use crate::palette::*;
use crate::tower::{Tower, TowerBuiltEvent, TowerPreview};
use crate::MouseWorldPos;

pub struct GoldPlugin;

impl Plugin for GoldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ModifySpawnerEvent>()
            .add_event::<PileCapEvent>()
            .add_event::<PileSpawnEvent>()
            .add_event::<PileRemoveEvent>()
            .add_event::<SpawnGoldEvent>()
            .add_startup_system(setup)
            .add_system(pile_input)
            .add_system(spawn_pile)
            .add_system(remove_pile)
            .add_system(generate_gold)
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

struct SpawnGoldEvent {
    position: Vec3,
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

pub struct PileSpawnEvent {
    pub coords: HexCoords,
    starting_gold: u32,
}

impl PileSpawnEvent {
    pub fn new(coords: HexCoords) -> Self {
        PileSpawnEvent {
            coords: coords,
            starting_gold: 0,
        }
    }
}

pub struct PileCapEvent {
    pub coords: HexCoords,
}

pub struct PileRemoveEvent {
    pub coords: HexCoords,
}

fn store_gold(
    mut commands: Commands,
    q_gold: Query<(Entity, &Transform, &Gold)>,
    mut q_pile: Query<(&Transform, &mut GoldPile, &Hex)>,
    mut ev_cap: EventWriter<PileCapEvent>,
) {
    for (gold_ent, gold_trans, _gold) in q_gold.iter() {
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

fn setup(mut ev_spawn: EventWriter<PileSpawnEvent>) {
    // spawn a pile at the center with some starting cash
    ev_spawn.send(PileSpawnEvent {
        coords: HexCoords::new(),
        starting_gold: 6,
    });
}

fn spawn_pile(
    mut commands: Commands,
    mut ev_spawn: EventReader<PileSpawnEvent>,
    q_hexes: Query<(Entity, &Hex), Without<TowerPreview>>,
) {
    // don't run before hexes exist
    // this preserves the event that is send frame ~1
    // until hexes exist on frame ~2
    // then on frame ~3 this runs
    // or maybe frame ~2 if this system happens to run after the hex spawn system
    if !q_hexes.is_empty() {
        for ev in ev_spawn.iter() {
            for (ent, hex) in q_hexes.iter() {
                if ev.coords.is_same(hex.coords) {
                    commands
                        .entity(ent)
                        .insert(GoldPile {
                            count: ev.starting_gold,
                            gold_cap: 500,
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    color: ORANGE,
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
                }
            }
        }
    }
}

fn remove_pile(
    mut commands: Commands,
    mut ev_remove: EventReader<PileRemoveEvent>,
    mut ev_spawn_gold: EventWriter<SpawnGoldEvent>,
    q_piles: Query<(Entity, &Children, &Transform, &Hex, &GoldPile)>,
    //mut q_child: Query<&mut Sprite>,
) {
    for ev in ev_remove.iter() {
        for (ent, children, trans, hex, pile) in q_piles.iter() {
            if ev.coords.is_same(hex.coords) {
                for _ in 0..pile.count {
                    ev_spawn_gold.send(SpawnGoldEvent {
                        position: trans.translation,
                    });
                }
                for &child in children {
                    //println!("despawning children");
                    // runs once
                    commands.entity(child).despawn();
                }

                commands
                    .entity(ent)
                    // didn't work
                    //.remove_children(children)
                    .remove::<GoldPile>();
            }
        }
    }
}

fn pile_input(
    input: Res<Input<KeyCode>>,
    mut ev_spawn: EventWriter<PileSpawnEvent>,
    mut ev_remove: EventWriter<PileRemoveEvent>,
    q_selection: Query<(&Transform, &Hex), With<Selection>>,
) {
    for (_trans, hex) in q_selection.iter() {
        if input.just_pressed(KeyCode::X) {
            ev_remove.send(PileRemoveEvent { coords: hex.coords });
        }
        if input.just_pressed(KeyCode::G) {
            ev_spawn.send(PileSpawnEvent::new(hex.coords));
        }
    }
}

fn generate_gold(
    mut q_gold_spawners: Query<(&Transform, &mut GoldSpawner)>,
    mut ev_gold_spawn: EventWriter<SpawnGoldEvent>,
    time: Res<Time>,
) {
    for (trans, mut spawner) in q_gold_spawners.iter_mut() {
        if spawner.timer.tick(time.delta()).just_finished() {
            ev_gold_spawn.send(SpawnGoldEvent {
                position: trans.translation,
            });
        }
    }
}

fn spawn_gold(mut commands: Commands, mut ev_gold_spawn: EventReader<SpawnGoldEvent>) {
    for ev in ev_gold_spawn.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: GOLD,
                    custom_size: Some(Vec2::new(8.0, 12.)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        x: ev.position.x,
                        y: ev.position.y,
                        z: 0.3,
                    },
                    ..default()
                },
                ..default()
            })
            .insert(Gold);
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
