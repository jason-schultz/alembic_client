use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::network::{NetworkEvent, TileData};
use crate::tile_textures::TileTextures;

#[derive(Resource, Default)]
pub struct WorldContext {
    pub current_zone_id: Option<String>,
    pub current_room_id: Option<String>,
    pub zone_width: u16,
    pub zone_height: u16,
    pub tile_entities: HashMap<(u16, u16), Entity>,
}

#[derive(Component)]
pub struct WorldTile {
    pub x: u16,
    pub y: u16,
}

pub const TILE_SIZE: f32 = 64.0;

pub fn handle_world_events(
    mut events: EventReader<NetworkEvent>,
    mut world_ctx: ResMut<WorldContext>,
    mut commands: Commands,
    tile_textures: Res<TileTextures>,
) {
    for event in events.read() {
        match event {
            NetworkEvent::ZoneInfo {
                zone_id,
                zone_name,
                width,
                height,
            } => {
                info!("Entering zone: {zone_name} ({width}x{height})");
                world_ctx.current_zone_id = Some(zone_id.clone());
                world_ctx.current_room_id = None;
                world_ctx.zone_width = *width;
                world_ctx.zone_height = *height;
            }

            NetworkEvent::RoomInfo {
                room_id,
                room_name,
                width,
                height,
            } => {
                info!("Entering room: {room_name} ({width}x{height})");
                world_ctx.current_room_id = Some(room_id.clone());
                world_ctx.zone_width = *width;
                world_ctx.zone_height = *height;
            }

            NetworkEvent::ViewportUpdate { tiles, .. } => {
                update_tiles(&mut commands, &mut world_ctx, tiles, &tile_textures);
            }

            _ => {}
        }
    }
}

/// Returns a textured sprite if the tile has a loaded handle, otherwise a
/// colored fallback so the game stays playable while assets are missing.
fn tile_sprite(asset_id: &str, tile_textures: &TileTextures) -> Sprite {
    if let Some(handle) = tile_textures.textures.get(asset_id) {
        Sprite {
            image: handle.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        }
    } else {
        Sprite {
            color: tile_color(asset_id),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        }
    }
}

fn update_tiles(
    commands: &mut Commands,
    world_ctx: &mut WorldContext,
    tiles: &[TileData],
    tile_textures: &TileTextures,
) {
    let mut seen = HashSet::new();

    for tile in tiles {
        let key = (tile.x, tile.y);
        seen.insert(key);
        let sprite = tile_sprite(&tile.asset_id, tile_textures);
        let x = tile.x as f32 * TILE_SIZE;
        let y = -(tile.y as f32 * TILE_SIZE);

        if let Some(&entity) = world_ctx.tile_entities.get(&key) {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).insert(sprite);
            } else {
                world_ctx.tile_entities.remove(&key);
                let new_entity = commands
                    .spawn((
                        sprite,
                        Transform::from_xyz(x, y, -1.0),
                        WorldTile { x: tile.x, y: tile.y },
                    ))
                    .id();
                world_ctx.tile_entities.insert(key, new_entity);
            }
        } else {
            let entity = commands
                .spawn((
                    sprite,
                    Transform::from_xyz(x, y, -1.0),
                    WorldTile { x: tile.x, y: tile.y },
                ))
                .id();
            world_ctx.tile_entities.insert(key, entity);
        }
    }

    let to_remove: Vec<(u16, u16)> = world_ctx
        .tile_entities
        .keys()
        .filter(|key| !seen.contains(key))
        .cloned()
        .collect();

    for key in to_remove {
        if let Some(entity) = world_ctx.tile_entities.remove(&key) {
            commands.entity(entity).despawn();
        }
    }
}

fn tile_color(asset_id: &str) -> Color {
    match asset_id {
        "grass_01" => Color::srgb(0.29, 0.62, 0.23),
        "grass_02" => Color::srgb(0.35, 0.68, 0.28),
        "dirt_01" => Color::srgb(0.55, 0.40, 0.22),
        "dirt_path_01" => Color::srgb(0.65, 0.50, 0.30),
        "sand_01" => Color::srgb(0.85, 0.78, 0.50),
        "snow_01" => Color::srgb(0.92, 0.95, 0.98),
        "water_01" => Color::srgb(0.13, 0.37, 0.75),
        "water_shallow_01" => Color::srgb(0.30, 0.60, 0.85),
        "path_01" => Color::srgb(0.68, 0.58, 0.42),
        "cobble_01" => Color::srgb(0.50, 0.48, 0.45),
        "road_01" => Color::srgb(0.42, 0.40, 0.38),
        "wall_stone_01" => Color::srgb(0.45, 0.43, 0.40),
        "wall_wood_01" => Color::srgb(0.45, 0.28, 0.12),
        "wall_brick_01" => Color::srgb(0.58, 0.22, 0.15),
        "floor_wood_01" => Color::srgb(0.60, 0.40, 0.18),
        "floor_stone_01" => Color::srgb(0.55, 0.53, 0.50),
        "floor_tile_01" => Color::srgb(0.72, 0.70, 0.68),
        "door_01" => Color::srgb(0.55, 0.35, 0.10),
        // Bright magenta — missing tiles are immediately visible during dev
        _ => Color::srgb(0.85, 0.10, 0.85),
    }
}
