use bevy::prelude::*;
use crate::{game::player::InGameEntity, ui::main_menu::GameState};

#[derive(Component)]
pub struct InGameUI;

#[derive(Component)]
pub struct ChatBox;

pub fn setup_in_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Game background (could be a tilemap later)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.3, 0.2), // Green-ish ground
            custom_size: Some(Vec2::new(2000.0, 2000.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -2.0),
        InGameUI,
    ));
    
    // Simple UI overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            InGameUI,
        ))
        .with_children(|parent| {
            // Top bar - character info
            parent
                .spawn(Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Character: Hero | Level: 1 | HP: 100/100"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Bottom chat/command area
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(150.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                    ChatBox,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Welcome to Alembic!\nUse WASD or Arrow keys to move.\nPress ESC to return to menu."),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });
        });
}

pub fn in_game_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}

pub fn cleanup_in_game(
    mut commands: Commands,
    ui_query: Query<Entity, With<InGameUI>>,
    game_query: Query<Entity, With<InGameEntity>>,
) {
    for entity in &ui_query {
        commands.entity(entity).despawn_recursive();
    }

    for entity in &game_query {
        commands.entity(entity).despawn_recursive();
    }
}