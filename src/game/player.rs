use super::animation::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

#[derive(Component)]
pub struct InGameEntity;

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load all zombie animations
    let animations = load_zombie_animations(&asset_server);

    // Get the first frame of idle animation
    let initial_frame = animations
        .get(&AnimationState::Idle)
        .and_then(|anim| anim.frames.first())
        .cloned()
        .unwrap_or_default();

    // Spawn player with animated sprite
    commands.spawn((
        Sprite {
            image: initial_frame,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(0.15)), // Scale down to 15%
        Player { speed: 150.0 },
        AnimatedSprite {
            current_state: AnimationState::Idle,
            animations,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        },
        InGameEntity,
    ));
}

pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut AnimatedSprite, &Player)>,
) {
    for (mut transform, mut animated, player) in &mut query {
        let mut direction = Vec3::ZERO;
        let mut moving = false;

        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
            moving = true;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
            moving = true;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
            moving = true;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
            moving = true;
        }

        // Handle attack
        if keyboard.just_pressed(KeyCode::Space) {
            animated.current_state = AnimationState::Attack;
            // Reset animation frame
            if let Some(animation) = animated.animations.get_mut(&AnimationState::Attack) {
                animation.current_frame = 0;
            }
        }

        // Check if attack animation finished
        let attack_finished = if animated.current_state == AnimationState::Attack {
            if let Some(animation) = animated.animations.get(&AnimationState::Attack) {
                animation.current_frame >= animation.frames.len() - 1
            } else {
                false
            }
        } else {
            false
        };

        // Update animation state based on movement
        if moving {
            if animated.current_state != AnimationState::Attack || attack_finished {
                animated.current_state = AnimationState::Walk;
            }

            if direction.length() > 0.0 {
                direction = direction.normalize();
                transform.translation += direction * player.speed * time.delta_secs();

                // Flip sprite based on direction
                if direction.x < 0.0 {
                    transform.scale.x = -0.15; // Face left
                } else if direction.x > 0.0 {
                    transform.scale.x = 0.15; // Face right
                }
            }
        } else {
            // Only return to idle if not attacking or attack finished
            if animated.current_state != AnimationState::Attack || attack_finished {
                animated.current_state = AnimationState::Idle;
            }
        }
    }
}

// Debug function to trigger hurt/dead animations
pub fn debug_animations(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut AnimatedSprite, With<Player>>,
) {
    for mut animated in &mut query {
        if keyboard.just_pressed(KeyCode::Digit1) {
            animated.current_state = AnimationState::Idle;
        }
        if keyboard.just_pressed(KeyCode::Digit2) {
            animated.current_state = AnimationState::Walk;
        }
        if keyboard.just_pressed(KeyCode::Digit3) {
            animated.current_state = AnimationState::Attack;
            if let Some(animation) = animated.animations.get_mut(&AnimationState::Attack) {
                animation.current_frame = 0;
            }
        }
        if keyboard.just_pressed(KeyCode::Digit4) {
            animated.current_state = AnimationState::Hurt;
            if let Some(animation) = animated.animations.get_mut(&AnimationState::Hurt) {
                animation.current_frame = 0;
            }
        }
        if keyboard.just_pressed(KeyCode::Digit5) {
            animated.current_state = AnimationState::Dead;
            if let Some(animation) = animated.animations.get_mut(&AnimationState::Dead) {
                animation.current_frame = 0;
            }
        }
    }
}
