use bevy::prelude::*;

use crate::character::{Character, load_characters_for_world};
use crate::network::connection::NetworkConnection;
use crate::ui::main_menu::GameState;
use crate::ui::world_select::SelectedWorld;

#[derive(Component)]
pub struct CharacterSelectUI;

#[derive(Component)]
pub enum CharacterSelectButton {
    Select(String),
    CreateNew,
    Back,
}

/// The character the player has chosen to play as.
/// Set when they click a character, read by InGame.
#[derive(Resource)]
pub struct SelectedCharacter(pub Character);

pub fn setup_character_select(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selected_world: Res<SelectedWorld>,
) {
    let characters = load_characters_for_world(&selected_world.world_id);

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        ImageNode {
            image: asset_server.load("Stontex.png"),
            ..default()
        },
        ZIndex(-1),
        CharacterSelectUI,
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            CharacterSelectUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!(
                    "Choose Your Character — {}",
                    selected_world.world_name
                )),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            if characters.is_empty() {
                parent.spawn((
                    Text::new("No characters yet. Create one to get started!"),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.65, 0.65, 0.65)),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));
            } else {
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    })
                    .with_children(|parent| {
                        for character in &characters {
                            spawn_character_row(parent, character);
                        }
                    });
            }

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_nav_button(
                        parent,
                        "Create New Character",
                        CharacterSelectButton::CreateNew,
                    );
                    spawn_nav_button(parent, "Back", CharacterSelectButton::Back);
                });
        });
}

fn spawn_character_row(parent: &mut ChildBuilder, character: &Character) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(520.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            CharacterSelectButton::Select(character.name.clone()),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(character.name.clone()),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                    ));
                    parent.spawn((
                        Text::new(format!(
                            "Level {} {} {}",
                            character.level,
                            character.race.name(),
                            character.class.name()
                        )),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.65, 0.75)),
                    ));
                });

            parent.spawn((
                Text::new(format!("Lv {}", character.level)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.75, 0.4)),
            ));
        });
}

fn spawn_nav_button(parent: &mut ChildBuilder, text: &str, button_type: CharacterSelectButton) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            button_type,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

pub fn character_select_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &CharacterSelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    network: Res<NetworkConnection>,
    selected_world: Res<SelectedWorld>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                match button {
                    CharacterSelectButton::Select(name) => {
                        let characters = load_characters_for_world(&selected_world.world_id);
                        if let Some(character) = characters.into_iter().find(|c| &c.name == name) {
                            commands.insert_resource(SelectedCharacter(character));
                            if let Err(e) = network.send_join_world(&selected_world.world_id) {
                                error!("Failed to send join world: {e}");
                            }
                            next_state.set(GameState::InGame);
                        }
                    }
                    CharacterSelectButton::CreateNew => {
                        next_state.set(GameState::CreateCharacter);
                    }
                    CharacterSelectButton::Back => {
                        next_state.set(GameState::WorldSelect);
                    }
                }
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        }
    }
}

pub fn cleanup_character_select(
    mut commands: Commands,
    query: Query<Entity, With<CharacterSelectUI>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
