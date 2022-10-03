use bevy::prelude::*;

use crate::StartSpawningEnemiesEvent;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(button_system)
            .add_startup_system(start_menu)
            .insert_resource(AcceptInput(false))
            .add_system(remove_start_menu)
            .add_system(allow_input);
    }
}

#[derive(Component)]
struct StartMenu;

pub struct AcceptInput(pub bool);

const NORMAL_BUTTOM: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTOM: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTOM: Color = Color::rgb(0.35, 0.75, 0.35);

fn button_system(
    mut q_interaction: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut q_text: Query<&mut Text>,
    mut ev_start: EventWriter<StartSpawningEnemiesEvent>,
) {
    for (interaction, mut color, children) in &mut q_interaction {
        let mut text = q_text.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTOM.into();
                println!("Button pressed");
                ev_start.send(StartSpawningEnemiesEvent);
                // don't let the player click through the menu
            }
            Interaction::Hovered => {
                text.sections[0].value = "Game".to_string();
                *color = HOVERED_BUTTOM.into();
            }
            Interaction::None => {
                text.sections[0].value = "Start".to_string();
                *color = NORMAL_BUTTOM.into();
            }
        }
    }
}

fn allow_input(
    mut ev_start: EventReader<StartSpawningEnemiesEvent>,
    mut accept: ResMut<AcceptInput>,
) {
    for _ev in ev_start.iter() {
        accept.0 = true;
    }
}

fn start_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::AZURE.into(),
            ..default()
        })
        .insert(StartMenu)
        .with_children(|root| {
            root.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(40.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::Center,
                    //align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    
                    ..default()
                },
                color: Color::NONE.into(),//:BEIGE.into(),
                ..default()
            })
            .with_children(|center| {
                center
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            // default is 1280x720
                            // 150/1280 = 11.7%
                            // 65/720 = 9%
                            // 12% of 40% = 30%
                            //size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                            size: Size::new(Val::Percent(30.0), Val::Percent(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        color: NORMAL_BUTTOM.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_section(
                            "Button",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        ));
                    });
                center.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(10.0), Val::Percent(10.0)),
                        margin: UiRect::new(Val::Auto, Val::Auto, Val::Percent(10.0), Val::Percent(10.0)),
                        ..default()
                    },
                    image: UiImage(asset_server.load("sprites/gold2.png")),
                    ..default()
                });
            });
        });
}

fn remove_start_menu(
    mut commands: Commands,
    mut ev_start: EventReader<StartSpawningEnemiesEvent>,
    q_menu: Query<Entity, With<StartMenu>>,
) {
    for _ev in ev_start.iter() {
        for ent in q_menu.iter() {
            commands.entity(ent).despawn_recursive();
        }
    }
}
