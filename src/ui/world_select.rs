use bevy::prelude::*;

use crate::network::{NetworkCommand, NetworkEvent};
use crate::ui::main_menu::GameState;

// ── Resources ─────────────────────────────────────────────────────────────────

/// Persists the user's world choice for downstream states to read.
#[derive(Resource)]
pub struct SelectedWorld {
    pub world_id: String,
    pub world_name: String,
}

#[derive(Resource, Default)]
pub struct AvailableWorlds {
    pub worlds: Vec<WorldEntry>,
    pub loaded: bool,
}

#[derive(Debug, Clone)]
pub struct WorldEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub player_count: u16,
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct WorldSelectUI;

#[derive(Component)]
pub enum WorldSelectButton {
    Join(String),
    Back,
}

#[derive(Component)]
pub struct WorldListContainer;

#[derive(Component)]
pub struct WorldSelectStatus;

// ── Systems ───────────────────────────────────────────────────────────────────

pub fn setup_world_select(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Seed a placeholder until the server sends a real WORLD_LIST packet.
    commands.insert_resource(AvailableWorlds {
        worlds: vec![WorldEntry {
            id: "main_story".to_string(),
            name: "The Shattered Crown".to_string(),
            description: "An epic journey to restore the fractured kingdom.".to_string(),
            player_count: 0,
        }],
        loaded: true,
    });

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
        WorldSelectUI,
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            WorldSelectUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Choose a World"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.7, 0.6)),
                WorldSelectStatus,
            ));

            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    min_width: Val::Px(600.0),
                    ..default()
                },
                WorldListContainer,
            ));

            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    WorldSelectButton::Back,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Back"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

/// Replaces the placeholder list when the server sends a real WORLD_LIST packet.
pub fn handle_world_list_event(
    mut events: EventReader<NetworkEvent>,
    mut worlds: ResMut<AvailableWorlds>,
) {
    for event in events.read() {
        if let NetworkEvent::WorldList { worlds: received } = event {
            worlds.worlds = received
                .iter()
                .map(|w| WorldEntry {
                    id: w.world_id.clone(),
                    name: w.world_name.clone(),
                    description: w.description.clone(),
                    player_count: w.player_count,
                })
                .collect();
            worlds.loaded = true;
        }
    }
}

/// Rebuilds the world list UI whenever `AvailableWorlds` changes.
pub fn update_world_list_ui(
    mut commands: Commands,
    worlds: Res<AvailableWorlds>,
    container_query: Query<Entity, With<WorldListContainer>>,
    mut status_query: Query<&mut Text, With<WorldSelectStatus>>,
) {
    if !worlds.is_changed() {
        return;
    }

    let Ok(container) = container_query.get_single() else {
        return;
    };
    commands.entity(container).despawn_descendants();

    if let Ok(mut text) = status_query.get_single_mut() {
        text.0 = String::new();
    }

    commands.entity(container).with_children(|parent| {
        if worlds.worlds.is_empty() {
            parent.spawn((
                Text::new("No worlds available on this server."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        } else {
            for world in &worlds.worlds {
                spawn_world_row(parent, world);
            }
        }
    });
}

fn spawn_world_row(parent: &mut ChildBuilder, world: &WorldEntry) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(600.0),
                height: Val::Px(90.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
            WorldSelectButton::Join(world.id.clone()),
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
                        Text::new(world.name.clone()),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.95)),
                    ));
                    parent.spawn((
                        Text::new(world.description.clone()),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.7)),
                    ));
                });

            parent.spawn((
                Text::new(format!("{} online", world.player_count)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.75, 0.4)),
            ));
        });
}

pub fn world_select_system(
    mut interaction_query: Query<
        (&Interaction, &WorldSelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    worlds: Res<AvailableWorlds>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut network_commands: EventWriter<NetworkCommand>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                match button {
                    WorldSelectButton::Join(world_id) => {
                        if let Some(world) = worlds.worlds.iter().find(|w| &w.id == world_id) {
                            commands.insert_resource(SelectedWorld {
                                world_id: world.id.clone(),
                                world_name: world.name.clone(),
                            });
                            next_state.set(GameState::DownloadingAssets);
                        }
                    }
                    WorldSelectButton::Back => {
                        network_commands.send(NetworkCommand::Disconnect);
                        next_state.set(GameState::ServerList);
                    }
                }
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        }
    }
}

pub fn cleanup_world_select(mut commands: Commands, query: Query<Entity, With<WorldSelectUI>>) {
    commands.remove_resource::<AvailableWorlds>();
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
