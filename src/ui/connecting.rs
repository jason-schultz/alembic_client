use bevy::prelude::*;

use crate::network::{NetworkCommand, NetworkEvent};
use crate::ui::Spinner;
use crate::ui::main_menu::GameState;

#[derive(Component)]
pub struct ConnectingUI;

#[derive(Component)]
pub struct ConnectingStatusLabel;

pub fn setup_connecting(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        ConnectingUI,
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(24.0),
                ..default()
            },
            ConnectingUI,
        ))
        .with_children(|parent| {
            // Animated spinner text
            parent.spawn((
                Text::new("|"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.9)),
                Spinner {
                    timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                    frame: 0,
                },
            ));

            // Status text — updated by handle_connecting_events
            parent.spawn((
                Text::new("Connecting..."),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                ConnectingStatusLabel,
            ));

            // Cancel button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(44.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.15, 0.15)),
                    CancelButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Cancel"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

#[derive(Component)]
pub struct CancelButton;

pub fn handle_connecting_events(
    mut network_events: EventReader<NetworkEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut label_query: Query<&mut Text, With<ConnectingStatusLabel>>,
) {
    for event in network_events.read() {
        match event {
            NetworkEvent::Connecting => {
                set_label(&mut label_query, "Connecting...");
            }
            NetworkEvent::Authenticated { .. } => {
                next_state.set(GameState::WorldSelect);
            }
            NetworkEvent::AuthFailed { reason } => {
                set_label(&mut label_query, &format!("Auth failed: {reason}"));
            }
            NetworkEvent::Disconnected { reason } => {
                set_label(&mut label_query, &format!("Disconnected: {reason}"));
            }
            _ => {}
        }
    }
}

pub fn connecting_cancel_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CancelButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut network_commands: EventWriter<NetworkCommand>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                network_commands.send(NetworkCommand::Disconnect);
                next_state.set(GameState::ServerList);
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.25, 0.15, 0.15)),
        }
    }
}

pub fn cleanup_connecting(mut commands: Commands, query: Query<Entity, With<ConnectingUI>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn set_label(query: &mut Query<&mut Text, With<ConnectingStatusLabel>>, text: &str) {
    if let Ok(mut label) = query.get_single_mut() {
        label.0 = text.to_string();
    }
}
