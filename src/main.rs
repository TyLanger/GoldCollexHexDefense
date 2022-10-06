#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::{App, ClearColor, Color, Msaa, NonSend, WindowDescriptor};
use bevy::window::WindowId;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
//use bevy_game::GamePlugin;
use std::io::Cursor;
use winit::window::Icon;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::camera::{RenderTarget, ScalingMode},
};
use palette::*; //{DARK_BLUE, LIGHT_BLUE, DARK_ORANGE, BLUE};
use std::env;

mod boids;
mod enemies;
mod gold;
mod hexes;
mod input;
mod palette;
mod tower;
mod tutorial;

fn main() {
    //env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .insert_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
        .add_plugin(hexes::HexPlugin)
        .add_plugin(tower::TowerPlugin)
        .add_plugin(gold::GoldPlugin)
        .add_plugin(enemies::EnemyPlugin)
        .add_plugin(boids::BoidsPlugin)
        .add_plugin(tutorial::TutorialPlugin)
        .insert_resource(MouseWorldPos(Vec2::ONE * 10000.0))
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Gold Collex Hex Defense".to_string(),
            canvas: Some("#bevy".to_owned()),
            ..Default::default()
        })
        .add_event::<StartSpawningEnemiesEvent>()
        .add_startup_system(setup)
        .add_startup_system(set_window_icon)
        .add_system(update_mouse_position)
        .run();
}

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

struct MouseWorldPos(Vec2);

pub struct StartSpawningEnemiesEvent;

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical(720.),
            ..default()
        },
        // default is 0.4 all
        // camera_2d: Camera2d {
        //     clear_color: ClearColorConfig::Custom(Color::rgb(0.278, 0.247, 0.202))
        // },
        ..default()
    });
}

fn update_mouse_position(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MouseWorldPos>,
) {
    let (camera, camera_transform) = q_camera.single();

    let win = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    if let Some(screen_pos) = win.cursor_position() {
        let window_size = Vec2::new(win.width() as f32, win.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coords)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        mouse_pos.0 = world_pos;
    }
}

// Sets the icon on windows and X11
fn set_window_icon(windows: NonSend<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();
    let icon_buf = Cursor::new(include_bytes!("../assets/textures/app_icon.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
