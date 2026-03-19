use bevy::prelude::*;

use crate::available_sprites::AvailableSprites;
use crate::character::*;
use crate::network::connection::NetworkConnection;
use crate::ui::character_select::SelectedCharacter;
use crate::ui::main_menu::GameState;
use crate::ui::world_select::SelectedWorld;

// ── Step tracking ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreationStep {
    #[default]
    Appearance,
    Identity,
    Attributes,
    Confirm,
}

impl CreationStep {
    fn title(&self) -> &str {
        match self {
            CreationStep::Appearance => "Choose Appearance",
            CreationStep::Identity => "Name & Background",
            CreationStep::Attributes => "Assign Attributes",
            CreationStep::Confirm => "Confirm Character",
        }
    }

    fn number(&self) -> usize {
        match self {
            CreationStep::Appearance => 1,
            CreationStep::Identity => 2,
            CreationStep::Attributes => 3,
            CreationStep::Confirm => 4,
        }
    }

    fn next(&self) -> Option<CreationStep> {
        match self {
            CreationStep::Appearance => Some(CreationStep::Identity),
            CreationStep::Identity => Some(CreationStep::Attributes),
            CreationStep::Attributes => Some(CreationStep::Confirm),
            CreationStep::Confirm => None,
        }
    }

    fn prev(&self) -> Option<CreationStep> {
        match self {
            CreationStep::Appearance => None,
            CreationStep::Identity => Some(CreationStep::Appearance),
            CreationStep::Attributes => Some(CreationStep::Identity),
            CreationStep::Confirm => Some(CreationStep::Attributes),
        }
    }
}

// ── Resources ─────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct CharacterCreation {
    pub step: CreationStep,
    pub name: String,
    pub class: Option<CharacterClass>,
    pub race: Option<CharacterRace>,
    pub sprite_id: Option<String>,
    pub points_remaining: u32,
    pub attributes: Attributes,
}

impl CharacterCreation {
    pub fn reset() -> Self {
        Self {
            points_remaining: 27,
            attributes: Attributes::default(),
            ..Default::default()
        }
    }

    pub fn is_race_selected(&self, race: CharacterRace) -> bool {
        self.race == Some(race)
    }

    pub fn is_class_selected(&self, class: CharacterClass) -> bool {
        self.class == Some(class)
    }

    fn can_advance(&self) -> (bool, &'static str) {
        match self.step {
            CreationStep::Appearance => (true, ""),
            CreationStep::Identity => {
                if self.name.is_empty() {
                    (false, "Please enter a character name.")
                } else if self.race.is_none() {
                    (false, "Please select a race.")
                } else if self.class.is_none() {
                    (false, "Please select a class.")
                } else {
                    (true, "")
                }
            }
            CreationStep::Attributes => (true, ""),
            CreationStep::Confirm => (true, ""),
        }
    }
}

#[derive(Resource)]
pub struct CursorBlink {
    pub timer: Timer,
    pub visible: bool,
}

impl Default for CursorBlink {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            visible: true,
        }
    }
}

pub fn blink_cursor(
    time: Res<Time>,
    mut blink: ResMut<CursorBlink>,
    creation: Res<CharacterCreation>,
    mut text_query: Query<&mut Text, With<NameInputField>>,
) {
    if creation.step != CreationStep::Identity {
        return;
    }

    blink.timer.tick(time.delta());
    if blink.timer.just_finished() {
        blink.visible = !blink.visible;
        if let Ok(mut text) = text_query.get_single_mut() {
            let cursor = if blink.visible { "_" } else { " " };
            text.0 = if creation.name.is_empty() {
                format!("Enter name...{}", cursor)
            } else {
                format!("{}{}", creation.name, cursor)
            }
        }
    }
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct CreateCharacterUI;

#[derive(Component)]
pub struct StepContent;

#[derive(Component)]
pub struct StepTitle;

#[derive(Component)]
pub struct StepDot(usize);

#[derive(Component)]
pub struct NextButton;

#[derive(Component)]
pub struct ValidationMsg;

#[derive(Component)]
pub struct NameInputField;

#[derive(Component)]
pub struct AttributeDisplay(pub AttributeType);

#[derive(Component)]
pub struct PointsDisplay;

#[derive(Component)]
pub struct SelectedSpriteLabel;

#[derive(Component)]
pub struct SelectedChoice;

#[derive(Component)]
pub enum WizardButton {
    SelectSprite(String),
    SelectRace(CharacterRace),
    SelectClass(CharacterClass),
    IncreaseAttr(AttributeType),
    DecreaseAttr(AttributeType),
    Next,
    Back,
    Cancel,
    Confirm,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl AttributeType {
    pub fn all() -> &'static [AttributeType] {
        &[
            AttributeType::Strength,
            AttributeType::Dexterity,
            AttributeType::Constitution,
            AttributeType::Intelligence,
            AttributeType::Wisdom,
            AttributeType::Charisma,
        ]
    }

    pub fn short(&self) -> &str {
        match self {
            AttributeType::Strength => "STR",
            AttributeType::Dexterity => "DEX",
            AttributeType::Constitution => "CON",
            AttributeType::Intelligence => "INT",
            AttributeType::Wisdom => "WIS",
            AttributeType::Charisma => "CHA",
        }
    }

    pub fn long(&self) -> &str {
        match self {
            AttributeType::Strength => "Strength",
            AttributeType::Dexterity => "Dexterity",
            AttributeType::Constitution => "Constitution",
            AttributeType::Intelligence => "Intelligence",
            AttributeType::Wisdom => "Wisdom",
            AttributeType::Charisma => "Charisma",
        }
    }

    pub fn get(&self, a: &Attributes) -> u32 {
        match self {
            AttributeType::Strength => a.strength,
            AttributeType::Dexterity => a.dexterity,
            AttributeType::Constitution => a.constitution,
            AttributeType::Intelligence => a.intelligence,
            AttributeType::Wisdom => a.wisdom,
            AttributeType::Charisma => a.charisma,
        }
    }

    pub fn set(&self, a: &mut Attributes, v: u32) {
        match self {
            AttributeType::Strength => a.strength = v,
            AttributeType::Dexterity => a.dexterity = v,
            AttributeType::Constitution => a.constitution = v,
            AttributeType::Intelligence => a.intelligence = v,
            AttributeType::Wisdom => a.wisdom = v,
            AttributeType::Charisma => a.charisma = v,
        }
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup_create_character(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    available: Res<AvailableSprites>,
) {
    commands.insert_resource(CharacterCreation::reset());

    // Full-screen background via UI ImageNode — scales with window
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
        CreateCharacterUI,
    ));

    // Dark overlay
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ZIndex(0),
        CreateCharacterUI,
    ));

    // Outer centering container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ZIndex(1),
            CreateCharacterUI,
        ))
        .with_children(|root| {
            // Card
            root.spawn((
                Node {
                    width: Val::Px(740.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(28.0)),
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.07, 0.07, 0.11, 0.96)),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_children(|card| {
                // Header row: title + step dots
                card.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(CreationStep::Appearance.title()),
                        TextFont {
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.90, 0.98)),
                        StepTitle,
                    ));

                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|dots| {
                        for i in 1usize..=4 {
                            dots.spawn((
                                Node {
                                    width: Val::Px(if i == 1 { 22.0 } else { 9.0 }),
                                    height: Val::Px(9.0),
                                    ..default()
                                },
                                BackgroundColor(if i == 1 {
                                    Color::srgb(0.55, 0.45, 0.85)
                                } else {
                                    Color::srgb(0.28, 0.28, 0.38)
                                }),
                                BorderRadius::all(Val::Px(5.0)),
                                StepDot(i),
                            ));
                        }
                    });
                });

                // Hairline divider
                divider(card);

                // Step content — swapped out each step
                card.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(340.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    StepContent,
                ))
                .with_children(|content| {
                    build_appearance_step(content, &available);
                });

                // Validation message
                card.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.45, 0.4)),
                    ValidationMsg,
                ));

                divider(card);

                // Nav row
                card.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|nav| {
                    // Left buttons
                    nav.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|left| {
                        small_btn(
                            left,
                            "Cancel",
                            WizardButton::Cancel,
                            Color::srgb(0.22, 0.10, 0.10),
                        );
                        small_btn(
                            left,
                            "← Back",
                            WizardButton::Back,
                            Color::srgb(0.14, 0.14, 0.20),
                        );
                    });

                    // Next / Confirm
                    nav.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(26.0), Val::Px(11.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.32, 0.25, 0.62)),
                        BorderRadius::all(Val::Px(5.0)),
                        WizardButton::Next,
                        NextButton,
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new("Next →"),
                            TextFont {
                                font_size: 15.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });
            });
        });
}

// ── Step builders ─────────────────────────────────────────────────────────────

fn build_appearance_step(parent: &mut ChildBuilder, sprites: &AvailableSprites) {
    parent.spawn((
        Text::new(
            "Pick how your character looks. This is optional — you can skip and use a default.",
        ),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.55, 0.55, 0.65)),
    ));

    parent.spawn((
        Text::new("No appearance selected"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.45, 0.75, 0.45)),
        SelectedSpriteLabel,
    ));

    if sprites.loading {
        parent.spawn((
            Text::new("Downloading sprites from server..."),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.55, 0.55, 0.65)),
        ));
        return;
    }

    if !sprites.loaded || sprites.sprites.is_empty() {
        parent.spawn((
            Text::new("No sprites available from this server."),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.5, 0.4)),
        ));
        return;
    }

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|grid| {
            for sprite in &sprites.sprites {
                let handle = sprite.handle.clone().unwrap_or_default();
                let id = sprite.id.clone();
                grid.spawn((
                    Button,
                    Node {
                        width: Val::Px(54.0),
                        height: Val::Px(72.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.11, 0.11, 0.17)),
                    BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    BorderRadius::all(Val::Px(4.0)),
                    WizardButton::SelectSprite(id),
                ))
                .with_children(|p| {
                    p.spawn((
                        ImageNode {
                            image: handle,
                            rect: Some(bevy::math::Rect {
                                min: Vec2::ZERO,
                                max: Vec2::new(48.0, 64.0),
                            }),
                            ..default()
                        },
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(61.0),
                            ..default()
                        },
                    ));
                });
            }
        });
}

fn build_identity_step(parent: &mut ChildBuilder, creation: &CharacterCreation) {
    // Name field
    parent.spawn((
        Text::new("Character Name"),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.5, 0.62)),
    ));

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(12.0)),
                margin: UiRect::bottom(Val::Px(16.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.10, 0.16)),
            BorderColor(Color::srgb(0.45, 0.40, 0.70)), // always highlighted — it's the active input
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|p| {
            let display = if creation.name.is_empty() {
                "Enter name...".to_string()
            } else {
                format!("{}_", creation.name) // underscore cursor
            };
            p.spawn((
                Text::new(display),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(if creation.name.is_empty() {
                    Color::srgb(0.35, 0.35, 0.45)
                } else {
                    Color::srgb(0.90, 0.90, 0.95)
                }),
                NameInputField,
            ));
        });

    // Race + Class columns
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(20.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|cols| {
            // Race
            cols.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new("Race"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.62)),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
                for race in CharacterRace::all() {
                    pill_button(
                        col,
                        race.name(),
                        None,
                        creation.is_race_selected(*race),
                        WizardButton::SelectRace(*race),
                    );
                }
            });

            // Class
            cols.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                flex_grow: 1.0,
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new("Class"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.62)),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ));
                for class in CharacterClass::all() {
                    pill_button(
                        col,
                        class.name(),
                        Some(class.description()),
                        creation.is_class_selected(*class),
                        WizardButton::SelectClass(*class),
                    );
                }
            });
        });
}

fn build_attributes_step(parent: &mut ChildBuilder, creation: &CharacterCreation) {
    // Points header
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new("Use the point-buy system to customise your stats (8–18)."),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.55, 0.65)),
            ));
            row.spawn((
                Text::new(format!("{} pts left", creation.points_remaining)),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.85, 0.55)),
                PointsDisplay,
            ));
        });

    // 2-column grid
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|cols| {
            let left = [
                AttributeType::Strength,
                AttributeType::Dexterity,
                AttributeType::Constitution,
            ];
            let right = [
                AttributeType::Intelligence,
                AttributeType::Wisdom,
                AttributeType::Charisma,
            ];

            for attrs in [&left[..], &right[..]] {
                cols.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|col| {
                    for &attr in attrs {
                        attr_row(col, attr, attr.get(&creation.attributes));
                    }
                });
            }
        });
}

fn build_confirm_step(parent: &mut ChildBuilder, creation: &CharacterCreation) {
    parent.spawn((
        Text::new("Review your character. Click Confirm to create them."),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgb(0.55, 0.55, 0.65)),
    ));

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.10, 0.16)),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|card| {
            summary_row(card, "Name", &creation.name);
            summary_row(
                card,
                "Race",
                &creation
                    .race
                    .map(|r| r.name().to_string())
                    .unwrap_or_else(|| "—".to_string()),
            );
            summary_row(
                card,
                "Class",
                &creation
                    .class
                    .map(|c| c.name().to_string())
                    .unwrap_or_else(|| "—".to_string()),
            );
            summary_row(
                card,
                "Appearance",
                &creation
                    .sprite_id
                    .as_deref()
                    .unwrap_or("Default")
                    .to_string(),
            );

            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.20, 0.20, 0.28)),
            ));

            // Attribute chips
            card.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|row| {
                for &attr in AttributeType::all() {
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        ..default()
                    })
                    .with_children(|chip| {
                        chip.spawn((
                            Text::new(format!("{}", attr.get(&creation.attributes))),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.82, 0.80, 0.95)),
                        ));
                        chip.spawn((
                            Text::new(attr.short()),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.45, 0.45, 0.55)),
                        ));
                    });
                }
            });
        });
}

// ── Systems ───────────────────────────────────────────────────────────────────

pub fn create_character_system(
    mut commands: Commands,
    mut interactions: Query<
        (&Interaction, &WizardButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut creation: ResMut<CharacterCreation>,
    mut next_state: ResMut<NextState<GameState>>,
    step_content: Query<Entity, With<StepContent>>,
    mut step_title: Query<&mut Text, With<StepTitle>>,
    mut dots: Query<(&StepDot, &mut BackgroundColor, &mut Node), Without<Button>>,
    mut validation: Query<&mut Text, (With<ValidationMsg>, Without<StepTitle>)>,
    mut sprite_label: Query<
        &mut Text,
        (
            With<SelectedSpriteLabel>,
            Without<StepTitle>,
            Without<ValidationMsg>,
        ),
    >,
    available: Res<AvailableSprites>,
    selected_world: Res<SelectedWorld>,
    network: Res<NetworkConnection>,
) {
    for (interaction, button, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                match button {
                    WizardButton::SelectSprite(id) => {
                        creation.sprite_id = Some(id.clone());
                        if let Ok(mut t) = sprite_label.get_single_mut() {
                            t.0 = format!("Selected: {}", id.replace('_', " "));
                        }
                    }
                    WizardButton::SelectRace(race) => {
                        creation.race = Some(*race);
                        clear_msg(&mut validation);
                        // Rebuild the step so the selected button highlights
                        refresh_step(&mut commands, &step_content, &creation, &available);
                    }
                    WizardButton::SelectClass(class) => {
                        creation.class = Some(*class);
                        clear_msg(&mut validation);
                        refresh_step(&mut commands, &step_content, &creation, &available);
                    }
                    WizardButton::IncreaseAttr(attr) => {
                        let v = attr.get(&creation.attributes);
                        if v < 18 && creation.points_remaining > 0 {
                            attr.set(&mut creation.attributes, v + 1);
                            creation.points_remaining -= 1;
                        }
                    }
                    WizardButton::DecreaseAttr(attr) => {
                        let v = attr.get(&creation.attributes);
                        if v > 8 {
                            attr.set(&mut creation.attributes, v - 1);
                            creation.points_remaining += 1;
                        }
                    }
                    WizardButton::Next => {
                        let (ok, msg) = creation.can_advance();
                        if !ok {
                            set_msg(&mut validation, msg);
                            return;
                        }
                        clear_msg(&mut validation);
                        if let Some(next) = creation.step.next() {
                            creation.step = next;
                            refresh_step(&mut commands, &step_content, &creation, &available);
                            refresh_header(&mut step_title, &mut dots, creation.step.number());
                            // Remove the next_btn block that was here
                        }
                    }
                    WizardButton::Back => {
                        clear_msg(&mut validation);
                        if let Some(prev) = creation.step.prev() {
                            creation.step = prev;
                            refresh_step(&mut commands, &step_content, &creation, &available);
                            refresh_header(&mut step_title, &mut dots, creation.step.number());
                        } else {
                            next_state.set(GameState::CharacterSelect);
                        }
                    }
                    WizardButton::Cancel => {
                        next_state.set(GameState::CharacterSelect);
                    }
                    WizardButton::Confirm => {
                        if let (Some(class), Some(race)) = (creation.class, creation.race) {
                            let character = Character {
                                name: creation.name.clone(),
                                class,
                                race,
                                attributes: creation.attributes,
                                level: 1,
                                experience: 0,
                                sprite_id: creation.sprite_id.clone(),
                                world_id: Some(selected_world.world_id.clone()),
                            };
                            let _ = save_character(&character);
                            commands.insert_resource(SelectedCharacter(character));
                            if let Err(e) = network.send_join_world(&selected_world.world_id) {
                                error!("Failed to send join world: {e}");
                            }
                            next_state.set(GameState::InGame);
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.26, 0.26, 0.36));
            }
            Interaction::None => {
                *color = BackgroundColor(match button {
                    WizardButton::Cancel => Color::srgb(0.22, 0.10, 0.10),
                    WizardButton::Next | WizardButton::Confirm => Color::srgb(0.32, 0.25, 0.62),
                    _ => Color::srgb(0.13, 0.13, 0.20),
                });
            }
        }
    }
}

/// Updates the Next button label and component when on the Confirm step.
pub fn sync_next_button(
    creation: Res<CharacterCreation>,
    mut btn_query: Query<(&mut WizardButton, &Children), With<NextButton>>,
    mut text_query: Query<&mut Text>,
) {
    if !creation.is_changed() {
        return;
    }
    for (mut btn, children) in &mut btn_query {
        let on_confirm = creation.step == CreationStep::Confirm;
        *btn = if on_confirm {
            WizardButton::Confirm
        } else {
            WizardButton::Next
        };
        if let Some(&child) = children.first() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = if on_confirm {
                    "✓ Create".to_string()
                } else {
                    "Next →".to_string()
                };
            }
        }
    }
}

pub fn handle_name_input(
    mut key_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    mut creation: ResMut<CharacterCreation>,
    mut text_query: Query<&mut Text, With<NameInputField>>,
) {
    if creation.step != CreationStep::Identity {
        key_events.clear();
        return;
    }

    for event in key_events.read() {
        if event.state != bevy::input::ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            bevy::input::keyboard::Key::Character(c) => {
                for ch in c.chars() {
                    if creation.name.len() < 24 {
                        creation.name.push(ch);
                    }
                }
            }
            bevy::input::keyboard::Key::Backspace => {
                creation.name.pop();
            }
            _ => {}
        }
    }

    if let Ok(mut text) = text_query.get_single_mut() {
        text.0 = if creation.name.is_empty() {
            "Enter name..._".to_string()
        } else {
            format!("{}_", creation.name)
        };
    }
}

pub fn update_attribute_displays(
    creation: Res<CharacterCreation>,
    mut attr_q: Query<(&mut Text, &AttributeDisplay)>,
    mut pts_q: Query<&mut Text, (With<PointsDisplay>, Without<AttributeDisplay>)>,
) {
    if !creation.is_changed() {
        return;
    }
    for (mut t, d) in &mut attr_q {
        t.0 = d.0.get(&creation.attributes).to_string();
    }
    if let Ok(mut t) = pts_q.get_single_mut() {
        t.0 = format!("{} pts left", creation.points_remaining);
    }
}

pub fn populate_sprite_selection(
    mut commands: Commands,
    available: Res<AvailableSprites>,
    creation: Res<CharacterCreation>,
    step_content: Query<Entity, With<StepContent>>,
) {
    if !available.is_changed() || !available.loaded {
        return;
    }
    if creation.step != CreationStep::Appearance {
        return;
    }
    let Ok(entity) = step_content.get_single() else {
        return;
    };
    commands.entity(entity).despawn_descendants();
    commands.entity(entity).with_children(|p| {
        build_appearance_step(p, &available);
    });
}

pub fn cleanup_create_character(
    mut commands: Commands,
    query: Query<Entity, With<CreateCharacterUI>>,
) {
    commands.remove_resource::<CharacterCreation>();
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn refresh_step(
    commands: &mut Commands,
    content: &Query<Entity, With<StepContent>>,
    creation: &CharacterCreation,
    sprites: &AvailableSprites,
) {
    let Ok(e) = content.get_single() else { return };
    commands.entity(e).despawn_descendants();
    commands.entity(e).with_children(|p| match creation.step {
        CreationStep::Appearance => build_appearance_step(p, sprites),
        CreationStep::Identity => build_identity_step(p, creation),
        CreationStep::Attributes => build_attributes_step(p, creation),
        CreationStep::Confirm => build_confirm_step(p, creation),
    });
}

fn refresh_header(
    title: &mut Query<&mut Text, With<StepTitle>>,
    dots: &mut Query<(&StepDot, &mut BackgroundColor, &mut Node), Without<Button>>,
    current: usize,
) {
    let titles = [
        "Choose Appearance",
        "Name & Background",
        "Assign Attributes",
        "Confirm Character",
    ];
    if let Ok(mut t) = title.get_single_mut() {
        t.0 = titles[current - 1].to_string();
    }
    for (dot, mut color, mut node) in dots.iter_mut() {
        if dot.0 == current {
            *color = BackgroundColor(Color::srgb(0.55, 0.45, 0.85));
            node.width = Val::Px(22.0);
        } else if dot.0 < current {
            *color = BackgroundColor(Color::srgb(0.38, 0.32, 0.58));
            node.width = Val::Px(9.0);
        } else {
            *color = BackgroundColor(Color::srgb(0.28, 0.28, 0.38));
            node.width = Val::Px(9.0);
        }
    }
}

fn set_msg(q: &mut Query<&mut Text, (With<ValidationMsg>, Without<StepTitle>)>, msg: &str) {
    if let Ok(mut t) = q.get_single_mut() {
        t.0 = msg.to_string();
    }
}

fn clear_msg(q: &mut Query<&mut Text, (With<ValidationMsg>, Without<StepTitle>)>) {
    if let Ok(mut t) = q.get_single_mut() {
        t.0 = String::new();
    }
}

fn divider(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.22, 0.22, 0.30)),
    ));
}

fn small_btn(parent: &mut ChildBuilder, label: &str, btn: WizardButton, bg: Color) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(9.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            BorderRadius::all(Val::Px(4.0)),
            btn,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.5, 0.5)),
            ));
        });
}

fn pill_button(
    parent: &mut ChildBuilder,
    label: &str,
    sub: Option<&str>,
    selected: bool,
    btn: WizardButton,
) {
    let border_color = if selected {
        Color::srgb(0.55, 0.45, 0.85)
    } else {
        Color::srgb(0.24, 0.24, 0.34)
    };
    let bg_color = if selected {
        Color::srgb(0.22, 0.18, 0.38)
    } else {
        Color::srgb(0.12, 0.12, 0.19)
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
            BorderRadius::all(Val::Px(4.0)),
            btn,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if selected {
                    Color::srgb(0.85, 0.80, 1.0)
                } else {
                    Color::srgb(0.84, 0.84, 0.92)
                }),
            ));
            if let Some(s) = sub {
                p.spawn((
                    Text::new(s),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.46, 0.46, 0.56)),
                ));
            }
        });
}

fn attr_row(parent: &mut ChildBuilder, attr: AttributeType, value: u32) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(7.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.10, 0.16)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(attr.long()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.70, 0.70, 0.82)),
                Node {
                    width: Val::Px(95.0),
                    ..default()
                },
            ));

            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|ctrl| {
                ctrl.spawn((
                    Button,
                    Node {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.28, 0.12, 0.12)),
                    BorderRadius::all(Val::Px(3.0)),
                    WizardButton::DecreaseAttr(attr),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("−"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.55, 0.55)),
                    ));
                });

                ctrl.spawn((
                    Text::new(value.to_string()),
                    TextFont {
                        font_size: 17.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.88, 0.86, 1.0)),
                    Node {
                        width: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    AttributeDisplay(attr),
                ));

                ctrl.spawn((
                    Button,
                    Node {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.28, 0.12)),
                    BorderRadius::all(Val::Px(3.0)),
                    WizardButton::IncreaseAttr(attr),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("+"),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.9, 0.55)),
                    ));
                });
            });
        });
}

fn summary_row(parent: &mut ChildBuilder, label: &str, value: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.62)),
            ));
            row.spawn((
                Text::new(value.to_string()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.86, 0.94)),
            ));
        });
}
