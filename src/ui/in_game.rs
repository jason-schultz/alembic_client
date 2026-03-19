use bevy::prelude::*;

use crate::game::player::InGameEntity;
use crate::game::world::{WorldContext, WorldTile};
use crate::network::NetworkCommand;
use crate::ui::main_menu::GameState;

#[derive(Component)]
pub struct InGameUI;

pub fn setup_in_game(mut commands: Commands) {
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
            // Top bar
            parent
                .spawn(Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Character: Hero | Level: 1 | HP: 100/100"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Bottom chat area
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
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(
                            "Welcome to Alembic!\nUse WASD or arrow keys to move.\nPress ESC to disconnect.",
                        ),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                });
        });
}

pub fn in_game_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut network_commands: EventWriter<NetworkCommand>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        network_commands.send(NetworkCommand::Disconnect);
        next_state.set(GameState::MainMenu);
    }
}

pub fn cleanup_in_game(
    mut commands: Commands,
    ui_query: Query<Entity, With<InGameUI>>,
    game_query: Query<Entity, With<InGameEntity>>,
    tile_query: Query<Entity, With<WorldTile>>,
    mut world_ctx: ResMut<WorldContext>,
) {
    world_ctx.tile_entities.clear(); // ← add this

    for entity in ui_query
        .iter()
        .chain(game_query.iter())
        .chain(tile_query.iter())
    {
        commands.entity(entity).despawn_recursive();
    }
}
