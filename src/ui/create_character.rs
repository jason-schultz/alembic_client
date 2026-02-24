use bevy::prelude::*;
use crate::character::*;
use crate::ui::main_menu::GameState;

#[derive(Component)]
pub struct CreateCharacterUI;

#[derive(Resource)]
pub struct CharacterCreation {
    pub name: String,
    pub class: Option<CharacterClass>,
    pub race: Option<CharacterRace>,
    pub points_remaining: u32,
    pub attributes: Attributes,
}

impl Default for CharacterCreation {
    fn default() -> Self {
        Self {
            name: String::new(),
            class: None,
            race: None,
            points_remaining: 27, // Point-buy system
            attributes: Attributes::default(),
        }
    }
}

#[derive(Component)]
pub enum CreateCharacterButton {
    SelectClass(CharacterClass),
    SelectRace(CharacterRace),
    IncreaseAttribute(AttributeType),
    DecreaseAttribute(AttributeType),
    Create,
    Cancel,
}

#[derive(Clone, Copy)]
pub enum AttributeType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl AttributeType {
    fn name(&self) -> &str {
        match self {
            AttributeType::Strength => "Strength",
            AttributeType::Dexterity => "Dexterity",
            AttributeType::Constitution => "Constitution",
            AttributeType::Intelligence => "Intelligence",
            AttributeType::Wisdom => "Wisdom",
            AttributeType::Charisma => "Charisma",
        }
    }

    fn get_value(&self, attrs: &Attributes) -> u32 {
        match self {
            AttributeType::Strength => attrs.strength,
            AttributeType::Dexterity => attrs.dexterity,
            AttributeType::Constitution => attrs.constitution,
            AttributeType::Intelligence => attrs.intelligence,
            AttributeType::Wisdom => attrs.wisdom,
            AttributeType::Charisma => attrs.charisma,
        }
    }

    fn set_value(&self, attrs: &mut Attributes, value: u32) {
        match self {
            AttributeType::Strength => attrs.strength = value,
            AttributeType::Dexterity => attrs.dexterity = value,
            AttributeType::Constitution => attrs.constitution = value,
            AttributeType::Intelligence => attrs.intelligence = value,
            AttributeType::Wisdom => attrs.wisdom = value,
            AttributeType::Charisma => attrs.charisma = value,
        }
    }
}

#[derive(Component)]
pub struct NameInputField;

#[derive(Component)]
pub struct AttributeDisplay(AttributeType);

#[derive(Component)]
pub struct PointsDisplay;

pub fn setup_create_character(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CharacterCreation::default());
    
    // Background
    commands.spawn((
        Sprite {
            image: asset_server.load("Stontex.png"),
            custom_size: Some(Vec2::new(1200.0, 800.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        CreateCharacterUI,
    ));
    
    // Main container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(40.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            CreateCharacterUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Create Character"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Content container
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(40.0),
                    ..default()
                })
                .with_children(|parent| {
                    // Left column - Name, Race, Class
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(20.0),
                            width: Val::Percent(50.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Name section
                            parent.spawn((
                                Text::new("Character Name:"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));

                            parent.spawn((
                                Text::new("(Type in console for now)"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                                NameInputField,
                            ));

                            // Race selection
                            parent.spawn((
                                Text::new("Select Race:"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                Node {
                                    margin: UiRect::top(Val::Px(20.0)),
                                    ..default()
                                },
                            ));

                            for race in CharacterRace::all() {
                                create_selection_button(
                                    parent,
                                    race.name(),
                                    CreateCharacterButton::SelectRace(*race),
                                );
                            }

                            // Class selection
                            parent.spawn((
                                Text::new("Select Class:"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                Node {
                                    margin: UiRect::top(Val::Px(20.0)),
                                    ..default()
                                },
                            ));

                            for class in CharacterClass::all() {
                                create_class_button(parent, *class);
                            }
                        });

                    // Right column - Attributes
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(15.0),
                            width: Val::Percent(50.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Attributes:"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));

                            parent.spawn((
                                Text::new("Points Remaining: 27"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                PointsDisplay,
                            ));

                            create_attribute_row(parent, AttributeType::Strength);
                            create_attribute_row(parent, AttributeType::Dexterity);
                            create_attribute_row(parent, AttributeType::Constitution);
                            create_attribute_row(parent, AttributeType::Intelligence);
                            create_attribute_row(parent, AttributeType::Wisdom);
                            create_attribute_row(parent, AttributeType::Charisma);
                        });
                });

            // Bottom buttons
            parent
                .spawn(Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    column_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    create_action_button(parent, "Create Character", CreateCharacterButton::Create);
                    create_action_button(parent, "Cancel", CreateCharacterButton::Cancel);
                });
        });
}

fn create_selection_button(
    parent: &mut ChildBuilder,
    text: &str,
    button_type: CreateCharacterButton,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(40.0),
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
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

fn create_class_button(parent: &mut ChildBuilder, class: CharacterClass) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(250.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            CreateCharacterButton::SelectClass(class),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(class.name()),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
            parent.spawn((
                Text::new(class.description()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn create_attribute_row(parent: &mut ChildBuilder, attr_type: AttributeType) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(15.0),
            ..default()
        })
        .with_children(|parent| {
            // Attribute name
            parent.spawn((
                Text::new(format!("{}: ", attr_type.name())),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    width: Val::Px(120.0),
                    ..default()
                },
            ));

            // Decrease button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
                    CreateCharacterButton::DecreaseAttribute(attr_type),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("-"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Value display
            parent.spawn((
                Text::new("10"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    width: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                AttributeDisplay(attr_type),
            ));

            // Increase button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.4, 0.2)),
                    CreateCharacterButton::IncreaseAttribute(attr_type),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("+"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

fn create_action_button(
    parent: &mut ChildBuilder,
    text: &str,
    button_type: CreateCharacterButton,
) {
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

pub fn create_character_system(
    mut interaction_query: Query<
        (&Interaction, &CreateCharacterButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut creation: ResMut<CharacterCreation>,
    mut next_state: ResMut<NextState<GameState>>,
    mut attr_displays: Query<(&mut Text, &AttributeDisplay)>,
    mut points_display: Query<&mut Text, (With<PointsDisplay>, Without<AttributeDisplay>)>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                
                match button {
                    CreateCharacterButton::SelectRace(race) => {
                        creation.race = Some(*race);
                        println!("Selected race: {:?}", race);
                    }
                    CreateCharacterButton::SelectClass(class) => {
                        creation.class = Some(*class);
                        println!("Selected class: {:?}", class);
                    }
                    CreateCharacterButton::IncreaseAttribute(attr_type) => {
                        let current = attr_type.get_value(&creation.attributes);
                        if current < 18 && creation.points_remaining > 0 {
                            attr_type.set_value(&mut creation.attributes, current + 1);
                            creation.points_remaining -= 1;
                            update_displays(&mut attr_displays, &mut points_display, &creation);
                        }
                    }
                    CreateCharacterButton::DecreaseAttribute(attr_type) => {
                        let current = attr_type.get_value(&creation.attributes);
                        if current > 8 {
                            attr_type.set_value(&mut creation.attributes, current - 1);
                            creation.points_remaining += 1;
                            update_displays(&mut attr_displays, &mut points_display, &creation);
                        }
                    }
                    CreateCharacterButton::Create => {
                        if let (Some(class), Some(race)) = (creation.class, creation.race) {
                            if !creation.name.is_empty() {
                                let character = Character {
                                    name: creation.name.clone(),
                                    class,
                                    race,
                                    attributes: creation.attributes,
                                    level: 1,
                                    experience: 0,
                                };
                                
                                if let Err(e) = save_character(&character) {
                                    println!("Error saving character: {}", e);
                                } else {
                                    println!("Character created: {:?}", character);
                                    next_state.set(GameState::LoadCharacter);
                                }
                            } else {
                                println!("Please enter a character name!");
                            }
                        } else {
                            println!("Please select race and class!");
                        }
                    }
                    CreateCharacterButton::Cancel => {
                        next_state.set(GameState::LoadCharacter);
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

fn update_displays(
    attr_displays: &mut Query<(&mut Text, &AttributeDisplay)>,
    points_display: &mut Query<&mut Text, (With<PointsDisplay>, Without<AttributeDisplay>)>,
    creation: &CharacterCreation,
) {
    // Update attribute displays
    for (mut text, attr_display) in attr_displays.iter_mut() {
        let value = attr_display.0.get_value(&creation.attributes);
        text.0 = value.to_string();
    }
    
    // Update points display
    if let Ok(mut text) = points_display.get_single_mut() {
        text.0 = format!("Points Remaining: {}", creation.points_remaining);
    }
}

pub fn handle_name_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut creation: ResMut<CharacterCreation>,
    mut text_query: Query<&mut Text, With<NameInputField>>,
) {
    // Simple text input - backspace and letters
    if keyboard.just_pressed(KeyCode::Backspace) {
        creation.name.pop();
    }
    
    // This is simplified - in production you'd want proper text input
    // For now, you can set the name via console commands or use a proper text input crate
    
    // Update display
    if let Ok(mut text) = text_query.get_single_mut() {
        if creation.name.is_empty() {
            text.0 = "(Type in console for now)".to_string();
        } else {
            text.0 = creation.name.clone();
        }
    }
}

pub fn cleanup_create_character(
    mut commands: Commands,
    query: Query<Entity, With<CreateCharacterUI>>,
) {
    commands.remove_resource::<CharacterCreation>();
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}