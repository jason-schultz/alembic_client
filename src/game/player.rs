use bevy::prelude::*;

use super::animation::{AnimatedSprite, AnimationState};
use crate::{
    available_sprites::AvailableSprites,
    game::{animation::load_animations_from_manifest, world::TILE_SIZE},
    network::{NetworkConnection, NetworkEvent},
    ui::character_select::SelectedCharacter,
};

/// Marker component for all entities that belong to an active game session.
/// Used by cleanup systems to despawn everything on exit.
#[derive(Component)]
pub struct InGameEntity;

#[derive(Resource)]
pub struct MovementState {
    pub timer: Timer,
    pub pending_confirmation: bool,
    pub last_facing: Option<u8>,
    pub pending_facing: Option<u8>,
    pub direction_changed: bool,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            pending_confirmation: false,
            last_facing: None,
            pending_facing: None,
            direction_changed: false,
        }
    }
}

/// The local player entity.
#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub player_id: String,
}

/// Server-authoritative position. Updated when the server sends SpawnPosition
/// or a position correction. The visual transform follows this.
#[derive(Component, Default)]
pub struct ServerPosition {
    pub zone_id: String,
    pub x: u16,
    pub y: u16,
    pub world_x: u32,
    pub world_y: u32,
    pub facing: u8,
}

#[derive(Component)]
pub struct PlayerMovement {
    pub visual_x: f32,
    pub visual_y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub is_moving: bool,
    pub last_direction: AnimationState,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            visual_x: 0.0,
            visual_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            is_moving: false,
            last_direction: AnimationState::WalkSouth,
        }
    }
}

pub fn spawn_player(
    mut commands: Commands,
    selected: Option<Res<SelectedCharacter>>,
    available_sprites: Res<AvailableSprites>,
) {
    let sprite_entry = selected
        .as_ref()
        .and_then(|sc| sc.0.sprite_id.as_ref())
        .and_then(|id| available_sprites.sprites.iter().find(|s| &s.id == id));

    let Some(sprite_entry) = sprite_entry else {
        warn!("No sprite selected or sprite not found — cannot spawn player");
        return;
    };

    let Some(sheet_handle) = sprite_entry.handle.clone() else {
        warn!("Sprite '{}' handle not yet loaded", sprite_entry.id);
        return;
    };

    let animations = load_animations_from_manifest(
        sheet_handle.clone(),
        sprite_entry.cell_width,
        sprite_entry.cell_height,
        &sprite_entry.animations,
    );

    let cell_w = sprite_entry.cell_width as f32;
    let cell_h = sprite_entry.cell_height as f32;

    let mut sprite = Sprite {
        image: sheet_handle,
        ..default()
    };
    sprite.rect = Some(bevy::math::Rect {
        min: Vec2::ZERO,
        max: Vec2::new(cell_w, cell_h),
    });

    commands.spawn((
        sprite,
        Transform::from_xyz(0.0, 16.0, 1.0).with_scale(Vec3::splat(2.0)),
        Player {
            speed: 150.0,
            player_id: String::new(),
        },
        AnimatedSprite {
            current_state: AnimationState::Idle,
            animations,
            timer: Timer::from_seconds(0.12, TimerMode::Repeating),
        },
        ServerPosition::default(),
        PlayerMovement::default(),
        InGameEntity,
    ));
}

/// Handles incoming `NetworkEvent`s that affect the player entity.
pub fn handle_network_events(
    mut events: EventReader<NetworkEvent>,
    mut player_query: Query<(
        &mut Player,
        &mut ServerPosition,
        &mut Transform,
        &mut PlayerMovement,
    )>,
    mut movement_state: ResMut<MovementState>,
) {
    for event in events.read() {
        match event {
            NetworkEvent::Authenticated { player_id, .. } => {
                if let Ok((mut player, _, _, _)) = player_query.get_single_mut() {
                    player.player_id = player_id.clone();
                }
            }

            NetworkEvent::SpawnPosition {
                zone_id,
                x,
                y,
                world_x,
                world_y,
                facing,
            } => {
                if let Ok((_, mut server_pos, mut transform, mut movement)) =
                    player_query.get_single_mut()
                {
                    server_pos.zone_id = zone_id.clone();
                    server_pos.x = *x;
                    server_pos.y = *y;
                    server_pos.world_x = *world_x;
                    server_pos.world_y = *world_y;
                    server_pos.facing = *facing;

                    let wx = *x as f32 * TILE_SIZE;
                    let wy = -(*y as f32 * TILE_SIZE);
                    transform.translation.x = wx;
                    transform.translation.y = wy;
                    movement.visual_x = wx;
                    movement.visual_y = wy;
                    movement.target_x = wx;
                    movement.target_y = wy;
                }
            }

            NetworkEvent::PositionConfirmed {
                zone_id,
                x,
                y,
                world_x,
                world_y,
                ..
            } => {
                if let Ok((_, mut server_pos, _, mut movement)) = player_query.get_single_mut() {
                    server_pos.zone_id = zone_id.clone();
                    server_pos.x = *x;
                    server_pos.y = *y;
                    server_pos.world_x = *world_x;
                    server_pos.world_y = *world_y;
                    movement.target_x = *x as f32 * TILE_SIZE;
                    movement.target_y = -(*y as f32 * TILE_SIZE);
                }
                movement_state.pending_confirmation = false;
                movement_state.pending_facing = movement_state.last_facing; // retry the last facing if the player is still holding a direction
            }

            NetworkEvent::PositionCorrection {
                x,
                y,
                world_x,
                world_y,
                ..
            } => {
                if let Ok((_, mut server_pos, mut transform, mut movement)) =
                    player_query.get_single_mut()
                {
                    server_pos.x = *x;
                    server_pos.y = *y;
                    server_pos.world_x = *world_x;
                    server_pos.world_y = *world_y;
                    let wx = *x as f32 * TILE_SIZE;
                    let wy = -(*y as f32 * TILE_SIZE);
                    movement.target_x = wx;
                    movement.target_y = wy;
                    if !movement.is_moving {
                        transform.translation.x = wx;
                        transform.translation.y = wy;
                        movement.visual_x = wx;
                        movement.visual_y = wy;
                    }
                }
                movement_state.pending_confirmation = false;
                movement_state.pending_facing = movement_state.last_facing; // retry the last facing if the player is still holding a direction
            }

            _ => {}
        }
    }
}

/// Handles local input for player movement.
/// Client predicts movement locally for responsiveness; the server will send
/// corrections if the move was invalid.
pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut movement_state: ResMut<MovementState>,
    mut query: Query<(
        &mut Transform,
        &mut AnimatedSprite,
        &mut PlayerMovement,
        &Player,
    )>,
) {
    for (mut transform, mut animated, mut movement, _player) in &mut query {
        if movement_state.direction_changed {
            movement.visual_x = movement.target_x;
            movement.visual_y = movement.target_y;
            transform.translation.x = movement.visual_x;
            transform.translation.y = movement.visual_y;
            movement_state.direction_changed = false;
        }

        let moving = keyboard.pressed(KeyCode::KeyW)
            || keyboard.pressed(KeyCode::ArrowUp)
            || keyboard.pressed(KeyCode::KeyS)
            || keyboard.pressed(KeyCode::ArrowDown)
            || keyboard.pressed(KeyCode::KeyA)
            || keyboard.pressed(KeyCode::ArrowLeft)
            || keyboard.pressed(KeyCode::KeyD)
            || keyboard.pressed(KeyCode::ArrowRight);

        movement.is_moving = moving;

        let attack_finished = animated.current_state == AnimationState::Attack
            && animated
                .animations
                .get(&AnimationState::Attack)
                .map(|a| a.current_frame >= a.frame_count() - 1)
                .unwrap_or(false);

        if keyboard.just_pressed(KeyCode::Space) {
            animated.current_state = AnimationState::Attack;
            if let Some(anim) = animated.animations.get_mut(&AnimationState::Attack) {
                anim.current_frame = 0;
            }
        }

        if moving {
            if animated.current_state != AnimationState::Attack || attack_finished {
                let walk_state = if keyboard.pressed(KeyCode::KeyW)
                    || keyboard.pressed(KeyCode::ArrowUp)
                {
                    AnimationState::WalkNorth
                } else if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
                    AnimationState::WalkSouth
                } else if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
                    AnimationState::WalkEast
                } else {
                    AnimationState::WalkWest
                };
                animated.current_state = walk_state;
                movement.last_direction = walk_state;
            }
        } else if animated.current_state != AnimationState::Attack || attack_finished {
            // Transition to the idle facing the last direction walked
            animated.current_state = match movement.last_direction {
                AnimationState::WalkNorth => AnimationState::IdleNorth,
                AnimationState::WalkEast  => AnimationState::IdleEast,
                AnimationState::WalkWest  => AnimationState::IdleWest,
                _                         => AnimationState::IdleSouth,
            };
        }

        if moving {
            let lerp_speed = 8.0;
            let dx = movement.target_x - movement.visual_x;
            let dy = movement.target_y - movement.visual_y;
            if dx.abs() < 1.0 && dy.abs() < 1.0 {
                movement.visual_x = movement.target_x;
                movement.visual_y = movement.target_y;
            } else {
                movement.visual_x += dx * lerp_speed * time.delta_secs();
                movement.visual_y += dy * lerp_speed * time.delta_secs();
            }
            transform.translation.x = movement.visual_x;
            transform.translation.y = movement.visual_y;
        }
    }
}

pub fn apply_player_sprite(
    mut player_query: Query<(&mut Sprite, &mut AnimatedSprite, &mut Transform), With<Player>>,
    selected: Option<Res<SelectedCharacter>>,
    available_sprites: Res<AvailableSprites>,
) {
    if !available_sprites.is_changed() {
        return;
    }

    let Some(selected) = selected else { return };
    let Some(sprite_id) = &selected.0.sprite_id else {
        return;
    };

    let sprite_entry = available_sprites
        .sprites
        .iter()
        .find(|s| &s.id == sprite_id);

    let Some(entry) = sprite_entry else {
        warn!("Sprite '{}' not found in available sprites", sprite_id);
        return;
    };
    let Some(handle) = entry.handle.clone() else {
        return;
    };
    let (cell_w, cell_h) = (entry.cell_width, entry.cell_height);
    let anim_defs = entry.animations.clone();

    if let Ok((mut sprite, mut animated, mut transform)) = player_query.get_single_mut() {
        sprite.image = handle.clone();
        sprite.rect = Some(bevy::math::Rect {
            min: Vec2::ZERO,
            max: Vec2::new(cell_w as f32, cell_h as f32),
        });
        animated.animations = load_animations_from_manifest(handle, cell_w, cell_h, &anim_defs);
        animated.current_state = AnimationState::Idle;
        // Fix scale for sheet sprites
        transform.scale = Vec3::splat(2.0);
        info!("Applied player sprite: {}", sprite_id);
    }
}

pub fn setup_movement(mut commands: Commands) {
    commands.insert_resource(MovementState::default());
}

pub fn send_movement(
    time: Res<Time>,
    mut movement_state: ResMut<MovementState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    network: Res<NetworkConnection>,
    player_query: Query<&ServerPosition, With<Player>>,
) {
    let facing = if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        Some(0u8)
    } else if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        Some(1u8)
    } else if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        Some(2u8)
    } else if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        Some(3u8)
    } else {
        None
    };

    // Don't send another packet until the server confirms the last one
    if movement_state.pending_confirmation {
        movement_state.last_facing = facing;
        return;
    }

    let direction_changed = facing != movement_state.last_facing;

    if direction_changed {
        let previous_facing = movement_state.last_facing;
        movement_state.last_facing = facing;
        movement_state.timer.reset();

        if facing.is_some() && previous_facing.is_some() {
            movement_state.direction_changed = true;
        }

        if let Some(facing_byte) = facing {
            let Ok(server_pos) = player_query.get_single() else {
                return;
            };
            if let Err(e) = network.send_player_move(server_pos.x, server_pos.y, facing_byte) {
                error!("Failed to send player move: {e}");
            }
            movement_state.pending_confirmation = true;
            movement_state.pending_facing = Some(facing_byte); // ← must set here
        }
        return;
    }

    // Same direction held — fire on timer
    movement_state.timer.tick(time.delta());
    if !movement_state.timer.just_finished() {
        return;
    }

    if let Some(facing_byte) = facing {
        let Ok(server_pos) = player_query.get_single() else {
            return;
        };
        if let Err(e) = network.send_player_move(server_pos.x, server_pos.y, facing_byte) {
            error!("Failed to send player move: {e}");
        }
        movement_state.pending_confirmation = true;
        movement_state.pending_facing = Some(facing_byte); // ← must also set here
    }
}

/// Debug: cycle animation states with number keys.
pub fn debug_animations(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut AnimatedSprite, With<Player>>,
) {
    for mut animated in &mut query {
        let state = if keyboard.just_pressed(KeyCode::Digit1) {
            Some(AnimationState::Idle)
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            Some(AnimationState::Walk)
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            Some(AnimationState::Attack)
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            Some(AnimationState::Hurt)
        } else if keyboard.just_pressed(KeyCode::Digit5) {
            Some(AnimationState::Dead)
        } else {
            None
        };

        if let Some(state) = state {
            animated.current_state = state;
            if let Some(anim) = animated.animations.get_mut(&state) {
                anim.current_frame = 0;
            }
        }
    }
}

pub fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    if let (Ok(player), Ok(mut camera)) = (player_query.get_single(), camera_query.get_single_mut())
    {
        camera.translation.x = player.translation.x.round();
        camera.translation.y = player.translation.y.round();
    }
}
