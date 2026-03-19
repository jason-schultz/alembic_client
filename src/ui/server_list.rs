use bevy::prelude::*;

use crate::network::NetworkCommand;
use crate::servers::{ServerEntry, ServerList};
use crate::ui::main_menu::GameState;

#[derive(Resource)]
pub struct ActiveServer {
    pub name: String,
    pub address: String,
    pub http_base: String,
}

#[derive(Component)]
pub struct ServerListUI;

#[derive(Component)]
pub enum ServerListButton {
    Connect(usize),
    Remove(usize),
    AddServer,
    Back,
}

/// Tracks UI state for the add-server form.
#[derive(Resource, Default)]
pub struct ServerListState {
    pub adding: bool,
    pub new_name: String,
    pub new_address: String,
    /// Index of server currently being connected to.
    pub connecting_to: Option<usize>,
}

/// Bevy resource wrapping the persisted server list.
#[derive(Resource)]
pub struct ServerListResource(pub ServerList);

pub fn setup_server_list(mut commands: Commands, asset_server: Res<AssetServer>) {
    let server_list = ServerList::load();
    commands.insert_resource(ServerListResource(server_list));
    commands.insert_resource(ServerListState::default());
    rebuild_server_list_ui(&mut commands, &asset_server);
}

fn rebuild_server_list_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
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
        ServerListUI,
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
            ServerListUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Select Server"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Server list container — populated by update_server_list_ui
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    min_width: Val::Px(500.0),
                    ..default()
                },
                ServerListContainer,
            ));

            // Status label
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.85, 0.7)),
                ServerListStatus,
            ));

            // Bottom buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_action_button(parent, "Add Server", ServerListButton::AddServer);
                    spawn_action_button(parent, "Back", ServerListButton::Back);
                });
        });
}

#[derive(Component)]
pub struct ServerListContainer;

#[derive(Component)]
pub struct ServerListStatus;

/// Rebuilds the server entry rows whenever the list changes.
pub fn update_server_list_ui(
    mut commands: Commands,
    server_list: Res<ServerListResource>,
    container_query: Query<Entity, With<ServerListContainer>>,
) {
    if !server_list.is_changed() {
        return;
    }

    let Ok(container) = container_query.get_single() else {
        return;
    };

    // Despawn old rows
    commands.entity(container).despawn_descendants();

    // Spawn new rows
    commands.entity(container).with_children(|parent| {
        if server_list.0.servers.is_empty() {
            parent.spawn((
                Text::new("No servers saved. Add one below."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        } else {
            for (i, server) in server_list.0.servers.iter().enumerate() {
                spawn_server_row(parent, i, server);
            }
        }
    });
}

fn spawn_server_row(parent: &mut ChildBuilder, index: usize, server: &ServerEntry) {
    parent
        .spawn((Node {
            width: Val::Px(500.0),
            height: Val::Px(70.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|parent| {
            // Main connect button — takes up most of the row
            parent
                .spawn((
                    Button,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    ServerListButton::Connect(index),
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
                                Text::new(server.name.clone()),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                            ));
                            parent.spawn((
                                Text::new(server.address.clone()),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.55, 0.55, 0.65)),
                            ));
                        });

                    if let Some(ref date) = server.last_connected {
                        parent.spawn((
                            Text::new(format!("Last: {date}")),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        ));
                    }
                });

            // Remove button — small X to the right
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::left(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.35, 0.12, 0.12)),
                    ServerListButton::Remove(index),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("✕"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.6, 0.6)),
                    ));
                });
        });
}

fn spawn_action_button(parent: &mut ChildBuilder, label: &str, button_type: ServerListButton) {
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
                Text::new(label),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

pub fn server_list_button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &ServerListButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut server_list: ResMut<ServerListResource>,
    mut state: ResMut<ServerListState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut network_commands: EventWriter<NetworkCommand>,
    mut status_query: Query<&mut Text, With<ServerListStatus>>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                match button {
                    ServerListButton::Connect(index) => {
                        if let Some(server) = server_list.0.servers.get(*index) {
                            let addr = server.address.clone();
                            let name = server.name.clone();
                            let host = addr.split(':').next().unwrap_or("127.0.0.1");
                            let http_base = format!("http://{}:8080", host);

                            commands.insert_resource(ActiveServer {
                                name: name.clone(),
                                address: addr.clone(),
                                http_base,
                            });

                            server_list.0.touch(&addr);
                            state.connecting_to = Some(*index);

                            if let Ok(mut text) = status_query.get_single_mut() {
                                text.0 = format!("Connecting to {name}...");
                            }

                            network_commands.send(NetworkCommand::Connect { addr });
                            next_state.set(GameState::Connecting);
                        }
                    }
                    ServerListButton::AddServer => {
                        // For now, add a hardcoded prompt.
                        // Full text input will come when we build proper input widgets.
                        // This adds a placeholder that the player can edit in servers.toml.
                        server_list.0.add("New Server", "0.0.0.0:7777");
                        if let Ok(mut text) = status_query.get_single_mut() {
                            text.0 =
                                "Added placeholder server. Edit servers.toml to change address."
                                    .to_string();
                        }
                    }
                    ServerListButton::Back => {
                        next_state.set(GameState::MainMenu);
                    }
                    ServerListButton::Remove(index) => {
                        server_list.0.remove(*index);
                        if let Ok(mut text) = status_query.get_single_mut() {
                            text.0 = "Server Removed.".to_string();
                        }
                    }
                }
            }
            Interaction::Hovered => {
                // Make remove button brighter red on hover
                *color = match button {
                    ServerListButton::Remove(_) => BackgroundColor(Color::srgb(0.55, 0.15, 0.15)),
                    _ => BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
                }
            }
            Interaction::None => {
                *color = match button {
                    ServerListButton::Remove(_) => BackgroundColor(Color::srgb(0.35, 0.12, 0.12)),
                    _ => BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                }
            }
        }
    }
}

pub fn cleanup_server_list(mut commands: Commands, query: Query<Entity, With<ServerListUI>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    // Keep ServerListResource alive — we need it in Connecting screen
}
