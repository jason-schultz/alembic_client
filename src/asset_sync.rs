use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};
use std::thread;

use crate::asset_cache::{fetch_world_manifest, sync_sprite_cache, sync_tile_cache};
use crate::available_sprites::{AvailableSprites, SpritesFailed, SpritesReady};
use crate::tile_textures::TileTextures;

/// Combined result sent back from the background download thread.
pub struct AssetSyncResult {
    pub sprites: Vec<(String, PathBuf)>,
    pub tiles: Vec<(String, PathBuf)>,
}

/// Channel used to receive download results from the background thread.
#[derive(Resource)]
pub struct AssetSyncChannel {
    pub rx: Mutex<mpsc::Receiver<Result<AssetSyncResult, String>>>,
}

/// Starts a world-specific asset download on a background thread.
/// Called directly from `setup_downloading_assets` — not a Bevy system.
pub fn start_world_asset_sync(
    commands: &mut Commands,
    world_id: &str,
    http_base: &str,
    server_addr: &str,
) {
    let (tx, rx) = mpsc::channel();

    let world_id = world_id.to_string();
    let http_base = http_base.to_string();
    let server_addr = server_addr.to_string();

    thread::spawn(move || {
        let result = match fetch_world_manifest(&http_base, &world_id) {
            Ok(manifest) => {
                let tiles =
                    sync_tile_cache(&http_base, &server_addr, &world_id, &manifest.tiles);
                let sprites = sync_sprite_cache(
                    &http_base,
                    &server_addr,
                    &world_id,
                    &manifest.sprites.characters,
                );
                Ok(AssetSyncResult { sprites, tiles })
            }
            Err(e) => Err(format!("Failed to fetch world manifest: {e}")),
        };
        let _ = tx.send(result);
    });

    commands.insert_resource(AssetSyncChannel { rx: Mutex::new(rx) });
}

/// Polls the background download thread and fires events when done.
/// Runs as a global Update system in all states.
pub fn poll_asset_sync(
    mut commands: Commands,
    channel: Option<ResMut<AssetSyncChannel>>,
    mut sprites: ResMut<AvailableSprites>,
    mut ready_events: EventWriter<SpritesReady>,
    mut failed_events: EventWriter<SpritesFailed>,
) {
    let Some(channel) = channel else { return };

    match channel.rx.lock().unwrap().try_recv() {
        Ok(Ok(result)) => {
            ready_events.send(SpritesReady {
                sprites: result.sprites,
                tiles: result.tiles,
            });
            sprites.loading = false;
            commands.remove_resource::<AssetSyncChannel>();
        }
        Ok(Err(reason)) => {
            failed_events.send(SpritesFailed {
                reason: reason.clone(),
            });
            sprites.loading = false;
            sprites.error = Some(reason);
            commands.remove_resource::<AssetSyncChannel>();
        }
        Err(mpsc::TryRecvError::Empty) => {}
        Err(mpsc::TryRecvError::Disconnected) => {
            sprites.loading = false;
            commands.remove_resource::<AssetSyncChannel>();
        }
    }
}

/// Loads all downloaded assets into Bevy's asset server.
/// Populates AvailableSprites (character sprites) and TileTextures (tile images).
/// Runs as a global Update system in all states.
pub fn handle_sprites_ready(
    mut events: EventReader<SpritesReady>,
    mut available: ResMut<AvailableSprites>,
    mut tile_textures: ResMut<TileTextures>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        available.sprites = event
            .sprites
            .iter()
            .map(|(id, path)| crate::available_sprites::AvailableSprite {
                id: id.clone(),
                local_path: path.clone(),
                handle: Some(asset_server.load(path.to_string_lossy().to_string())),
            })
            .collect();
        available.loaded = true;
        available.loading = false;

        tile_textures.textures = event
            .tiles
            .iter()
            .map(|(id, path)| {
                (
                    id.clone(),
                    asset_server.load(path.to_string_lossy().to_string()),
                )
            })
            .collect();
        tile_textures.loaded = true;

        info!(
            "Loaded {} character sprites and {} tiles from cache",
            available.sprites.len(),
            tile_textures.textures.len()
        );
    }
}
