use bevy::sprite::collide_aabb::collide;
use bevy::utils::Duration;
use bevy::{prelude::*, utils::FloatOrd};
use rand::prelude::*;

use crate::{gold::Gold, palette::*};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemyEvent>()
            .add_startup_system(setup)
            .add_system(generate_enemies)
            .add_system(spawn_enemy)
            .add_system(move_enemies)
            .add_system(grab_gold)
            .add_system(escape);
    }
}

#[derive(Component)]
struct Enemy {
    has_gold: bool,
}

impl Enemy {
    fn new() -> Self {
        Enemy { has_gold: false }
    }
}

#[derive(Component)]
struct EnemySpawner {
    timer: Timer,
}

struct SpawnEnemyEvent {
    position: Vec3,
}

fn setup(mut commands: Commands) {
    commands.spawn().insert(EnemySpawner {
        timer: Timer::new(Duration::from_secs(2), true),
    });
}

fn generate_enemies(
    time: Res<Time>,
    mut ev_spawn_enemy: EventWriter<SpawnEnemyEvent>,
    mut q_spawner: Query<&mut EnemySpawner>,
) {
    for mut spawner in q_spawner.iter_mut() {
        if spawner.timer.tick(time.delta()).finished() {
            for _ in 0..10 {
                let mut rng = rand::thread_rng();
                let spawn_pos = Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0))
                    .normalize_or_zero()
                    * 500.;

                ev_spawn_enemy.send(SpawnEnemyEvent {
                    position: spawn_pos.extend(0.3),
                })
            }
        }
    }
}

fn spawn_enemy(mut commands: Commands, mut ev_spawn_enemy: EventReader<SpawnEnemyEvent>) {
    for ev in ev_spawn_enemy.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: CRIMSON,
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform {
                    translation: ev.position,
                    ..default()
                },
                ..default()
            })
            .insert(Enemy::new());
    }
}

fn move_enemies(
    mut q_enemies: Query<(&mut Transform, &Enemy)>,
    q_gold: Query<&Transform, (With<Gold>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (mut trans, enemy) in q_enemies.iter_mut() {
        let mut dir = Vec3::new(0.0, 0.0, 0.0) - trans.translation;

        if enemy.has_gold {
            dir = trans.translation - Vec3::ZERO;
        } else {
            let direction = q_gold
                .iter()
                .min_by_key(|target_transform| {
                    FloatOrd(Vec3::distance(
                        target_transform.translation,
                        trans.translation,
                    ))
                })
                .map(|closest_target| closest_target.translation - trans.translation);

            if let Some(direction) = direction {
                dir = direction;
            }
        }

        dir.z = 0.0;

        trans.translation += dir.normalize_or_zero() * 100. * time.delta_seconds();
    }
}

fn grab_gold(
    mut commands: Commands,
    mut q_enemies: Query<(Entity, &Transform, &mut Enemy)>,
    mut q_gold: Query<(Entity, &mut Transform), (Without<Enemy>, With<Gold>)>,
) {
    // when you grab the gold, run away
    // directly away from 0,0 ?
    // remove the gold?
    // add something to the enemy so they don't pick up more gold?
    for (ent, mut gold_trans) in q_gold.iter_mut() {
        for (e_ent, e_trans, mut enemy) in q_enemies.iter_mut() {
            if let Some(_) = collide(
                gold_trans.translation,
                Vec2::new(8., 12.),
                e_trans.translation,
                Vec2::new(15., 15.),
            ) {
                println!("Grabbed a gold");
                enemy.has_gold = true;
                commands.entity(ent).remove::<Gold>();

                commands.entity(e_ent).add_child(ent);
                gold_trans.translation = Vec3::new(0.0, 0.0, 0.1);
                break;
            }
        }
    }
}

fn escape(mut commands: Commands, q_enemies: Query<(Entity, &Enemy, &Transform)>) {
    for (ent, enemy, trans) in q_enemies.iter() {
        if enemy.has_gold {
            if trans.translation.distance(Vec3::ZERO) > 700.0 {
                // escaped
                println!("Escaped");
                commands.entity(ent).despawn_recursive();
            }
        }
    }
}
