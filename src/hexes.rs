use bevy::{
    prelude::*,
    sprite::{collide_aabb::collide, MaterialMesh2dBundle},
};

use crate::MouseWorldPos;

const DEG_TO_RAD: f32 = 0.01745;
const HEX_SPACING: f32 = 0.86602540378;

pub struct HexPlugin;

impl Plugin for HexPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HexSpawnEvent>()
            .add_startup_system(setup)
            .add_system(spawn_hex)
            .add_system(highlight_hex.before(select_hex))
            .add_system(select_hex);
            //.add_system(colour_neighbours.after(highlight_hex));
        // the way selection is added and removed can mess these up
        // but I don't really know how to guarantee it.
        // are queries set at the start of the frame
        // or are they made when that system runs?
    }
}

struct HexSpawnEvent {
    position: Vec2,
    radius: f32,
    coords: HexCoords,
}

#[derive(Component)]
pub struct Hex {
    radius: f32,
    pub coords: HexCoords,
}

#[derive(Component)]
pub struct Selection;

fn highlight_hex(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q_hex: Query<(Entity, &mut Handle<ColorMaterial>, Option<&Selection>), With<Hex>>,
) {
    for (ent, color_handle, select) in q_hex.iter_mut() {
        if let Some(_) = select {
            let mut color_mat = materials.get_mut(&color_handle).unwrap();
            color_mat.color = Color::SILVER;

            commands.entity(ent).insert(Selection);
        } else {
            let mut color_mat = materials.get_mut(&color_handle).unwrap();
            color_mat.color = Color::MIDNIGHT_BLUE;
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
        if let Some(_) = collide(
            mouse.0.extend(0.0),
            Vec2::new(0.1, 0.1),
            trans.translation,
            Vec2::new(1.6 * hex.radius, 1.8 * hex.radius),
        ) {
            return;
        } else {
            commands.entity(ent).remove::<Selection>();
        }
    }

    for (ent, trans, hex) in q_hex.iter() {
        // bounding box for hexes is close enough
        // 1.6 so you don't select multiple.
        if let Some(_) = collide(
            mouse.0.extend(0.0),
            Vec2::new(0.1, 0.1),
            trans.translation,
            Vec2::new(1.6 * hex.radius, 1.6 * hex.radius),
        ) {
            commands.entity(ent).insert(Selection);
            // println!("Selected: {:?}", hex.coords);
            // println!("Neighbours: {:?}", hex.coords.print_neighbours());
            return;
        }
    }
}

fn colour_neighbours(
    q_selection: Query<&Hex, With<Selection>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_hexes: Query<(&Hex, &mut Handle<ColorMaterial>)>,
) {
    for select_hex in q_selection.iter() {
        let neighbours = select_hex.coords.get_neighbours();
        for (hex, color_handle) in q_hexes.iter() {
            for n in neighbours.iter() {
                if n.is_same(hex.coords) {
                    // is a neighbour
                    let mut color_mat = materials.get_mut(&color_handle).unwrap();
                    color_mat.color = Color::rgb(0.9, 0.9, 0.9);
                }
            }
        }
    }
}

fn setup(ev_spawn: EventWriter<HexSpawnEvent>) {
    spawn_hexes(ev_spawn, 7, 20., Vec2::ZERO);
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
                coords: HexCoords {
                    level: 0,
                    position: 0,
                    offset: 0,
                },
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

        for j in 0..i {
            ev_spawn.send(HexSpawnEvent {
                position: center + top_offset,
                radius: hex_radius,
                coords: HexCoords {
                    level: i,
                    position: 0,
                    offset: j,
                },
            });
            top_offset += Vec2::new(0.5 * hex_radius + hex_radius, -HEX_SPACING * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + top_right_offset,
                radius: hex_radius,
                coords: HexCoords {
                    level: i,
                    position: 1,
                    offset: j,
                },
            });
            top_right_offset += Vec2::new(0., -HEX_SPACING * 2. * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + bottom_right_offset,
                radius: hex_radius,
                coords: HexCoords {
                    level: i,
                    position: 2,
                    offset: j,
                },
            });
            bottom_right_offset +=
                Vec2::new(-(0.5 * hex_radius + hex_radius), -HEX_SPACING * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + bottom_offset,
                radius: hex_radius,
                coords: HexCoords {
                    level: i,
                    position: 3,
                    offset: j,
                },
            });
            bottom_offset += Vec2::new(-(0.5 * hex_radius + hex_radius), HEX_SPACING * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + bottom_left_offset,
                radius: hex_radius,
                coords: HexCoords {
                    level: i,
                    position: 4,
                    offset: j,
                },
            });
            bottom_left_offset += Vec2::new(0., HEX_SPACING * 2. * hex_radius);

            ev_spawn.send(HexSpawnEvent {
                position: center + top_left_offset,
                radius: hex_radius,
                coords: HexCoords {
                    level: i,
                    position: 5,
                    offset: j,
                },
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
        let (position, radius, coords) = (ev.position, ev.radius, ev.coords);

        //println!("Spawn: {:?}", coords);

        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(radius, 6).into())
                    .into(),
                material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                transform: Transform::from_translation(position.extend(0.1))
                    .with_rotation(Quat::from_rotation_z(30.0 * DEG_TO_RAD)),
                ..default()
            })
            .insert(Hex { radius, coords });
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HexCoords {
    // center tile is 0,0
    // tile above is 1,0
    // tile to the top-right is 1,1
    // these coords probably don't do anything
    // your neighbours are the 2 on the same level, +-1 position
    // but then 1-2 from the inner
    // and 2-3 from the outer
    level: u32,
    position: u32,
    offset: u32,
}

impl HexCoords {
    pub fn is_same(self, other: HexCoords) -> bool {
        return self.level == other.level
            && self.position == other.position
            && self.offset == other.offset;
    }

    fn _print_neighbours(self) {
        let n = self.get_neighbours();
        println!("Neighbours: {:?}", n);
    }

    pub fn get_neighbours(self) -> [HexCoords; 6] {
        match self.level {
            0 => {
                //println!("Center tile");
                let a = HexCoords {
                    level: 1,
                    position: 0,
                    offset: 0,
                };
                let b = HexCoords {
                    level: 1,
                    position: 1,
                    offset: 0,
                };
                let c = HexCoords {
                    level: 1,
                    position: 2,
                    offset: 0,
                };
                let d = HexCoords {
                    level: 1,
                    position: 3,
                    offset: 0,
                };
                let e = HexCoords {
                    level: 1,
                    position: 4,
                    offset: 0,
                };
                let f = HexCoords {
                    level: 1,
                    position: 5,
                    offset: 0,
                };
                return [a, b, c, d, e, f];
            }
            1 => {
                // same level
                let a = HexCoords {
                    level: 1,
                    position: (self.position + 1) % 6,
                    offset: 0,
                };
                let b = HexCoords {
                    level: 1,
                    position: (self.position + 5) % 6,
                    offset: 0,
                };
                // inner (center)
                let c = HexCoords {
                    level: 0,
                    position: 0,
                    offset: 0,
                };
                // outer
                let d = HexCoords {
                    level: 2,
                    position: (self.position + 5) % 6,
                    offset: 1,
                };
                let e = HexCoords {
                    level: 2,
                    position: self.position,
                    offset: 0,
                };
                let f = HexCoords {
                    level: 2,
                    position: self.position,
                    offset: 1,
                };
                // println!(
                //     "Neighbours are: {:?}; {:?}; {:?}; {:?}; {:?}; {:?};",
                //     a, b, c, d, e, f
                // );
                return [a, b, c, d, e, f];
            }
            _ => {
                let a;
                let b;
                let c;
                let d;
                let e;
                let f;

                let small_offset = 1;
                let large_offset = self.level - 1;

                if self.offset == 0 {
                    // 1 inner, 2 same, 3 outer
                    // same level
                    a = HexCoords {
                        level: self.level,
                        position: self.position,
                        offset: small_offset,
                    };
                    b = HexCoords {
                        level: self.level,
                        position: (self.position + 5) % 6,
                        offset: large_offset,
                    };
                    // inner (center)
                    c = HexCoords {
                        level: self.level - 1,
                        position: self.position,
                        offset: 0,
                    };
                    // outer
                    d = HexCoords {
                        level: self.level + 1,
                        position: (self.position + 5) % 6,
                        offset: self.level,
                    };
                    e = HexCoords {
                        level: self.level + 1,
                        position: self.position,
                        offset: 0,
                    };
                    f = HexCoords {
                        level: self.level + 1,
                        position: self.position,
                        offset: 1,
                    };

                    return [a, b, c, d, e, f];
                } else {
                    // 2 inner, 2 same, 2 outer
                    // example: (2, 0, 1); (3, 2, 2)
                    // same level
                    a = HexCoords {
                        level: self.level,
                        position: self.position,
                        offset: self.offset - 1,
                    };
                    if self.offset == large_offset {
                        // i.e. offsets don't get bigger
                        // most be the next 'corner'
                        b = HexCoords {
                            level: self.level,
                            position: (self.position + 1) % 6,
                            offset: 0,
                        };
                    } else {
                        b = HexCoords {
                            level: self.level,
                            position: self.position,
                            offset: self.offset + 1,
                        };
                    }

                    // inner (center)
                    c = HexCoords {
                        level: self.level - 1,
                        position: self.position,
                        offset: self.offset - 1,
                    };
                    if self.offset == large_offset {
                        d = HexCoords {
                            level: self.level - 1,
                            position: (self.position + 1) % 6,
                            offset: 0,
                        };
                    } else {
                        d = HexCoords {
                            level: self.level - 1,
                            position: self.position,
                            offset: self.offset,
                        };
                    }

                    // outer
                    e = HexCoords {
                        level: self.level + 1,
                        position: self.position,
                        offset: self.offset,
                    };
                    f = HexCoords {
                        level: self.level + 1,
                        position: self.position,
                        offset: self.offset + 1,
                    };

                    return [a, b, c, d, e, f];
                }
            }
        }
    }
}
