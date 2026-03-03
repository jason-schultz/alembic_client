mod auth;
mod character;
mod game;
mod network;
mod ui;

use auth::AuthToken;
use bevy::prelude::*;
use game::animation::*;
use game::player::*;
use network::{NetworkConnection, handshake_system, poll_network};
use ui::connect_server::*;
use ui::create_character::*;
use ui::in_game::*;
use ui::load_character::*;
use ui::main_menu::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alembic - MUD Client".to_string(),
                resolution: (1200., 800.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(NetworkConnection::default())
        .insert_resource(AuthToken {
            token: "your-token-here".into(),
        })
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        // Main Menu
        .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
        .add_systems(
            Update,
            (main_menu_system, animate_torches).run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
        // Load Character
        .add_systems(OnEnter(GameState::LoadCharacter), setup_load_character)
        .add_systems(
            Update,
            load_character_system.run_if(in_state(GameState::LoadCharacter)),
        )
        .add_systems(OnExit(GameState::LoadCharacter), cleanup_load_character)
        // Create Character
        .add_systems(OnEnter(GameState::CreateCharacter), setup_create_character)
        .add_systems(
            Update,
            (create_character_system, handle_name_input)
                .run_if(in_state(GameState::CreateCharacter)),
        )
        .add_systems(OnExit(GameState::CreateCharacter), cleanup_create_character)
        // Connect to Server
        .add_systems(OnEnter(GameState::ConnectToServer), setup_connect_server)
        .add_systems(
            Update,
            (connect_server_system, update_connection).run_if(in_state(GameState::ConnectToServer)),
        )
        .add_systems(OnExit(GameState::ConnectToServer), cleanup_connect_server)
        // In Game
        .add_systems(OnEnter(GameState::InGame), (setup_in_game, spawn_player))
        .add_systems(
            Update,
            (
                move_player,
                animate_sprites,
                in_game_input,
                debug_animations,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(OnExit(GameState::InGame), cleanup_in_game)
        .add_systems(Update, poll_network)
        .add_systems(Update, handshake_system.after(poll_network))
        .run();
}
