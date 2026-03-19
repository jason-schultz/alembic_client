use bevy::prelude::*;

use crate::asset_sync::{AssetSyncChannel, start_world_asset_sync};
use crate::available_sprites::{AvailableSprites, SpritesFailed, SpritesReady};
use crate::character::load_characters_for_world;
use crate::ui::Spinner;
use crate::ui::main_menu::GameState;
use crate::ui::server_list::ActiveServer;
use crate::ui::world_select::SelectedWorld;
use crate::tile_textures::TileTextures;


// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct DownloadingAssetsUI;

#[derive(Component)]
pub struct DownloadStatusLabel;

#[derive(Component)]
pub struct DownloadBackButton;

// ── Systems ───────────────────────────────────────────────────────────────────

pub fn setup_downloading_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selected_world: Res<SelectedWorld>,
    active_server: Option<Res<ActiveServer>>,
    mut sprites: ResMut<AvailableSprites>,
    mut tile_textures: ResMut<TileTextures>,
) {
    sprites.clear();
    sprites.loading = true;
    tile_textures.textures.clear();
    tile_textures.loaded = false;

    let (http_base, server_addr) = active_server
        .as_ref()
        .map(|s| (s.http_base.clone(), s.address.clone()))
        .unwrap_or_else(|| {
            (
                "http://127.0.0.1:8080".to_string(),
                "127.0.0.1:7777".to_string(),
            )
        });

    start_world_asset_sync(
        &mut commands,
        &selected_world.world_id,
        &http_base,
        &server_addr,
    );

    // Background
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
        DownloadingAssetsUI,
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
            DownloadingAssetsUI,
        ))
        .with_children(|parent| {
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

            parent.spawn((
                Text::new(format!(
                    "Downloading assets for '{}'...",
                    selected_world.world_name
                )),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                DownloadStatusLabel,
            ));

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
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    DownloadBackButton,
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

/// Transitions to CharacterSelect or CreateCharacter once assets are ready.
/// Checks whether any characters already exist for this world to decide which.
pub fn handle_download_events(
    mut sprites_ready: EventReader<SpritesReady>,
    mut sprites_failed: EventReader<SpritesFailed>,
    mut next_state: ResMut<NextState<GameState>>,
    selected_world: Res<SelectedWorld>,
    mut label_query: Query<&mut Text, With<DownloadStatusLabel>>,
) {
    for _ in sprites_ready.read() {
        let characters = load_characters_for_world(&selected_world.world_id);
        if characters.is_empty() {
            next_state.set(GameState::CreateCharacter);
        } else {
            next_state.set(GameState::CharacterSelect);
        }
        return;
    }

    for event in sprites_failed.read() {
        if let Ok(mut label) = label_query.get_single_mut() {
            label.0 = format!("Download failed: {}", event.reason);
        }
    }
}

pub fn download_back_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DownloadBackButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::WorldSelect);
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.35)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        }
    }
}

pub fn cleanup_downloading_assets(
    mut commands: Commands,
    query: Query<Entity, With<DownloadingAssetsUI>>,
) {
    // Drop the channel so a still-running background thread's result is ignored.
    commands.remove_resource::<AssetSyncChannel>();
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
