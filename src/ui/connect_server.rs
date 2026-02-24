use bevy::prelude::*;
use crate::ui::main_menu::GameState;

#[derive(Component)]
pub struct ConnectServerUI;

#[derive(Component)]
pub enum ConnectButton {
    Connect,
    Back,
}

#[derive(Resource)]
pub struct ConnectionState {
    pub connecting: bool,
    pub progress: f32,
    pub timer: Timer,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            connecting: false,
            progress: 0.0,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

pub fn setup_connect_server(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ConnectionState::default());
    
    // Background
    commands.spawn((
        Sprite {
            image: asset_server.load("Stontex.png"),
            custom_size: Some(Vec2::new(1200.0, 800.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        ConnectServerUI,
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
            ConnectServerUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Connect to Server"),
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

            // Server info (fake for now)
            parent.spawn((
                Text::new("Server: localhost:4000\nStatus: Ready"),
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

            // Progress bar container (hidden initially)
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(30.0),
                        margin: UiRect::bottom(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    Visibility::Hidden,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
                    ));
                });

            // Buttons
            parent
                .spawn(Node {
                    column_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    create_button(parent, "Connect", ConnectButton::Connect);
                    create_button(parent, "Back", ConnectButton::Back);
                });
        });
}

fn create_button(parent: &mut ChildBuilder, text: &str, button_type: ConnectButton) {
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

#[derive(Component)]
struct ProgressBar;

#[derive(Component)]
struct ProgressBarFill;

pub fn connect_server_system(
    mut interaction_query: Query<
        (&Interaction, &ConnectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut connection: ResMut<ConnectionState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                match button {
                    ConnectButton::Connect => {
                        println!("Connecting to server...");
                        connection.connecting = true;
                        connection.progress = 0.0;
                    }
                    ConnectButton::Back => {
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

pub fn update_connection(
    time: Res<Time>,
    mut connection: ResMut<ConnectionState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if connection.connecting {
        connection.timer.tick(time.delta());
        
        if connection.timer.just_finished() {
            connection.progress += 10.0;
            
            if connection.progress >= 100.0 {
                println!("Connected! Loading game...");
                connection.connecting = false;
                next_state.set(GameState::InGame);
            } else {
                println!("Loading assets... {}%", connection.progress);
            }
        }
    }
}

pub fn cleanup_connect_server(
    mut commands: Commands,
    query: Query<Entity, With<ConnectServerUI>>,
) {
    commands.remove_resource::<ConnectionState>();
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}