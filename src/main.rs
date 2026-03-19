mod asset_cache;
mod asset_sync;
mod auth;
mod available_sprites;
mod character;
mod game;
mod network;
mod servers;
mod tile_textures;
mod ui;

use auth::AuthToken;
use bevy::prelude::*;
use game::animation::animate_sprites;
use game::player::{debug_animations, handle_network_events, move_player, spawn_player};
use game::world::{WorldContext, handle_world_events};
use network::NetworkPlugin;
use tile_textures::TileTextures;
use ui::character_select::{
    character_select_system, cleanup_character_select, setup_character_select,
};
use ui::connecting::{
    cleanup_connecting, connecting_cancel_system, handle_connecting_events, setup_connecting,
};
use ui::create_character::{
    cleanup_create_character, create_character_system, setup_create_character,
};
use ui::downloading_assets::{
    cleanup_downloading_assets, download_back_system, handle_download_events,
    setup_downloading_assets,
};
use ui::in_game::{cleanup_in_game, in_game_input, setup_in_game};
use ui::main_menu::{
    GameState, animate_torches, cleanup_main_menu, main_menu_system, reset_camera, setup_camera,
    setup_main_menu,
};
use ui::server_list::{
    cleanup_server_list, server_list_button_system, setup_server_list, update_server_list_ui,
};
use ui::world_select::{
    cleanup_world_select, handle_world_list_event, setup_world_select, update_world_list_ui,
    world_select_system,
};

use crate::asset_sync::{handle_sprites_ready, poll_asset_sync};
use crate::available_sprites::{AvailableSprites, SpritesFailed, SpritesReady};
use crate::game::player::{apply_player_sprite, camera_follow, send_movement, setup_movement};
use crate::ui::animate_spinner;
use crate::ui::create_character::{
    CursorBlink, blink_cursor, handle_name_input, populate_sprite_selection, sync_next_button,
    update_attribute_displays,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alembic".to_string(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(AuthToken {
            token: "your-token-here".into(),
        })
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(WorldContext::default())
        .insert_resource(AvailableSprites::default())
        .insert_resource(TileTextures::default())
        .insert_resource(CursorBlink::default())
        .add_event::<SpritesReady>()
        .add_event::<SpritesFailed>()
        .add_plugins(NetworkPlugin)
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        // Global systems — run in all states
        .add_systems(Update, (poll_asset_sync, handle_sprites_ready))
        // Main Menu
        .add_systems(OnEnter(GameState::MainMenu), (setup_main_menu, reset_camera))
        .add_systems(
            Update,
            (main_menu_system, animate_torches).run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        // Server List
        .add_systems(OnEnter(GameState::ServerList), setup_server_list)
        .add_systems(
            Update,
            (server_list_button_system, update_server_list_ui)
                .run_if(in_state(GameState::ServerList)),
        )
        .add_systems(OnExit(GameState::ServerList), cleanup_server_list)
        // Connecting
        .add_systems(OnEnter(GameState::Connecting), setup_connecting)
        .add_systems(
            Update,
            (animate_spinner, handle_connecting_events, connecting_cancel_system)
                .run_if(in_state(GameState::Connecting)),
        )
        .add_systems(OnExit(GameState::Connecting), cleanup_connecting)
        // World Select
        .add_systems(OnEnter(GameState::WorldSelect), setup_world_select)
        .add_systems(
            Update,
            (world_select_system, handle_world_list_event, update_world_list_ui)
                .run_if(in_state(GameState::WorldSelect)),
        )
        .add_systems(OnExit(GameState::WorldSelect), cleanup_world_select)
        // Downloading Assets
        .add_systems(OnEnter(GameState::DownloadingAssets), setup_downloading_assets)
        .add_systems(
            Update,
            (animate_spinner, handle_download_events, download_back_system)
                .run_if(in_state(GameState::DownloadingAssets)),
        )
        .add_systems(OnExit(GameState::DownloadingAssets), cleanup_downloading_assets)
        // Character Select
        .add_systems(OnEnter(GameState::CharacterSelect), setup_character_select)
        .add_systems(
            Update,
            character_select_system.run_if(in_state(GameState::CharacterSelect)),
        )
        .add_systems(OnExit(GameState::CharacterSelect), cleanup_character_select)
        // Create Character
        .add_systems(OnEnter(GameState::CreateCharacter), setup_create_character)
        .add_systems(
            Update,
            (
                create_character_system,
                handle_name_input,
                populate_sprite_selection,
                update_attribute_displays,
                sync_next_button,
                blink_cursor,
            )
                .run_if(in_state(GameState::CreateCharacter)),
        )
        .add_systems(OnExit(GameState::CreateCharacter), cleanup_create_character)
        // In Game
        .add_systems(
            OnEnter(GameState::InGame),
            (setup_in_game, spawn_player, setup_movement),
        )
        .add_systems(
            Update,
            (
                send_movement,
                move_player.after(send_movement),
                animate_sprites,
                in_game_input,
                debug_animations,
                handle_network_events,
                handle_world_events,
                camera_follow.after(move_player),
                apply_player_sprite,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(OnExit(GameState::InGame), cleanup_in_game)
        .run();
}
