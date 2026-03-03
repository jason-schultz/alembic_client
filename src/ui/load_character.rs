use crate::character::*;
use crate::ui::main_menu::GameState;
use bevy::prelude::*;

#[derive(Component)]
pub struct LoadCharacterUI;

#[derive(Component)]
pub enum LoadCharacterButton {
    SelectCharacter(String),
    CreateNew,
    Back,
}

pub fn setup_load_character(mut commands: Commands, asset_server: Res<AssetServer>) {
    let characters = load_all_characters();

    // Background
    commands.spawn((
        Sprite {
            image: asset_server.load("Stontex.png"),
            custom_size: Some(Vec2::new(1200.0, 800.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        LoadCharacterUI,
    ));

    // UI Container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            LoadCharacterUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Load Character"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            if characters.is_empty() {
                // No characters message
                parent.spawn((
                    Text::new("No saved characters found"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    Node {
                        margin: UiRect::bottom(Val::Px(30.0)),
                        ..default()
                    },
                ));
            } else {
                // Character list
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        margin: UiRect::bottom(Val::Px(30.0)),
                        ..default()
                    })
                    .with_children(|parent| {
                        for character in characters {
                            create_character_button(parent, &character);
                        }
                    });
            }

            // Buttons container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    create_nav_button(
                        parent,
                        "Create New Character",
                        LoadCharacterButton::CreateNew,
                    );
                    create_nav_button(parent, "Back", LoadCharacterButton::Back);
                });
        });
}

fn create_character_button(parent: &mut ChildBuilder, character: &Character) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(400.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            LoadCharacterButton::SelectCharacter(character.name.clone()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!(
                    "{} - Level {} {} {}",
                    character.name,
                    character.level,
                    character.race.name(),
                    character.class.name()
                )),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

fn create_nav_button(parent: &mut ChildBuilder, text: &str, button_type: LoadCharacterButton) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
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

pub fn load_character_system(
    mut interaction_query: Query<
        (&Interaction, &LoadCharacterButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                match button {
                    LoadCharacterButton::SelectCharacter(name) => {
                        println!("Loading character: {}", name);
                        // TODO: Load character and transition to game
                        next_state.set(GameState::InGame);
                    }
                    LoadCharacterButton::CreateNew => {
                        next_state.set(GameState::CreateCharacter);
                    }
                    LoadCharacterButton::Back => {
                        next_state.set(GameState::MainMenu);
                    }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.35));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
        }
    }
}

pub fn cleanup_load_character(mut commands: Commands, query: Query<Entity, With<LoadCharacterUI>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
