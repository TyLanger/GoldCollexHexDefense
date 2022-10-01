use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, collide_aabb::collide}};

use crate::MouseWorldPos;

const DEG_TO_RAD: f32 = 0.01745;
const HEX_SPACING: f32 = 0.86602540378;

pub struct HexPlugin;

impl Plugin for HexPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HexSpawnEvent>()
            .add_startup_system(setup)
            .add_system(spawn_hex)
            .add_system(highlight_hex)
            .add_system(select_hex);
    }
}

struct HexSpawnEvent {
    position: Vec2,
    radius: f32,
}

#[derive(Component)]
struct Hex {
    radius: f32,
}

#[derive(Component)]
struct Selection;

fn highlight_hex(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_hex: Query<(Entity, &mut Handle<ColorMaterial>, Option<&Selection>), With<Hex>>,
) {

    for (ent, color_handle, select) in q_hex.iter_mut() {
        if let Some(_) = select{
            
            let mut color_mat = materials.get_mut(&color_handle).unwrap();
            color_mat.color = Color::SILVER;
            
            commands.entity(ent).insert(Selection);
            
        } else {
            let mut color_mat = materials.get_mut(&color_handle).unwrap();
            color_mat.color = Color::TURQUOISE;
        }
    }
}

fn select_hex(
    mut commands: Commands,
    q_hex: Query<(Entity, &Transform, &Hex)>,
    q_selection: Query<(Entity, &Transform, &Hex), With<Selection>>,
    mouse: Res<MouseWorldPos>,
) {
    for (ent, trans, hex) in q_selection.iter() {
        // does the current selection still count?
        // if colliding with 2, choose the one you already chose last frame
        if let Some(_) = collide(mouse.0.extend(0.0), Vec2::new(0.1, 0.1), trans.translation, Vec2::new(1.6 * hex.radius, 1.6 * hex.radius)) {
            return;
        } else {
            commands.entity(ent).remove::<Selection>();
        }
    }

    for (ent, trans, hex) in q_hex.iter() {
        // bounding box for hexes is close enough
        // 1.6 so you don't select multiple.
        if let Some(_) = collide(mouse.0.extend(0.0), Vec2::new(0.1, 0.1), trans.translation, Vec2::new(1.6 * hex.radius, 1.6 * hex.radius)) {
            commands.entity(ent).insert(Selection);
            return;
        }
    }
}


fn setup(ev_spawn: EventWriter<HexSpawnEvent>) {
    spawn_hexes(ev_spawn, 4, 20., Vec2::ZERO);
}

fn spawn_hexes(
    mut ev_spawn: EventWriter<HexSpawnEvent>,
    grid_radius: u32,
    hex_radius: f32,
    center: Vec2,
) {
    for i in 0..=grid_radius {
        if i == 0 {
            ev_spawn.send(HexSpawnEvent {
                position: center,
                radius: hex_radius,
            });
            continue;
        }

        let j32 = (i) as f32;
        let mut top_offset = Vec2::new(0., HEX_SPACING * 2. * hex_radius * j32);
        let mut top_right_offset = Vec2::new(
            (0.5 * hex_radius + hex_radius) * j32,
            HEX_SPACING * hex_radius * j32,
        );
        let mut bottom_right_offset = Vec2::new(
            (0.5 * hex_radius + hex_radius) * j32,
            -HEX_SPACING * hex_radius * j32,
        );
        let mut bottom_offset = Vec2::new(0., -HEX_SPACING * 2. * hex_radius * j32);
        let mut bottom_left_offset = Vec2::new(
            -(0.5 * hex_radius + hex_radius) * j32,
            -HEX_SPACING * hex_radius * j32,
        );
        let mut top_left_offset = Vec2::new(
            -(0.5 * hex_radius + hex_radius) * j32,
            HEX_SPACING * hex_radius * j32,
        );

        for _ in 0..i {
            ev_spawn.send(HexSpawnEvent {
                position: center + top_offset,
                radius: hex_radius,
            });
            top_offset += Vec2::new(0.5 * hex_radius + hex_radius, -HEX_SPACING * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + top_right_offset,
                radius: hex_radius,
            });
            top_right_offset += Vec2::new(0., -HEX_SPACING * 2. * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + bottom_right_offset,
                radius: hex_radius,
            });
            bottom_right_offset +=
                Vec2::new(-(0.5 * hex_radius + hex_radius), -HEX_SPACING * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + bottom_offset,
                radius: hex_radius,
            });
            bottom_offset += Vec2::new(-(0.5 * hex_radius + hex_radius), HEX_SPACING * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + bottom_left_offset,
                radius: hex_radius,
            });
            bottom_left_offset += Vec2::new(0., HEX_SPACING * 2. * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + top_left_offset,
                radius: hex_radius,
            });
            top_left_offset += Vec2::new(0.5 * hex_radius + hex_radius, HEX_SPACING * hex_radius);
        }
    }
}

fn spawn_hex(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ev_spawn: EventReader<HexSpawnEvent>,
) {
    for ev in ev_spawn.iter() {
        let (position, radius) = (ev.position, ev.radius);

        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::RegularPolygon::new(radius, 6).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
            transform: Transform::from_translation(position.extend(0.))
                .with_rotation(Quat::from_rotation_z(30.0 * DEG_TO_RAD)),
            ..default()
        })
        .insert(Hex { radius });
    }
}
