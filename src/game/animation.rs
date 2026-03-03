use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationState {
    Idle,
    Walk,
    Attack,
    Hurt,
    Dead,
}

#[derive(Component)]
pub struct AnimatedSprite {
    pub current_state: AnimationState,
    pub animations: HashMap<AnimationState, Animation>,
    pub timer: Timer,
}

#[derive(Clone)]
pub struct Animation {
    pub frames: Vec<Handle<Image>>,
    pub current_frame: usize,
    pub frame_time: f32,
}

impl Animation {
    pub fn new(frames: Vec<Handle<Image>>, frame_time: f32) -> Self {
        Self {
            frames,
            current_frame: 0,
            frame_time,
        }
    }
}

pub fn load_zombie_animations(
    asset_server: &Res<AssetServer>,
) -> HashMap<AnimationState, Animation> {
    let mut animations = HashMap::new();

    // Load idle animation (9 frames)
    let idle_frames: Vec<Handle<Image>> = (0..9)
        .map(|i| asset_server.load(format!("ZombieOGA/idle/__Zombie01_Idle_{:03}.png", i)))
        .collect();
    animations.insert(AnimationState::Idle, Animation::new(idle_frames, 0.1));

    // Load walk animation (9 frames)
    let walk_frames: Vec<Handle<Image>> = (0..9)
        .map(|i| asset_server.load(format!("ZombieOGA/walk/__Zombie01_Walk_{:03}.png", i)))
        .collect();
    animations.insert(AnimationState::Walk, Animation::new(walk_frames, 0.1));

    // Load attack animation (7 frames)
    let attack_frames: Vec<Handle<Image>> = (0..7)
        .map(|i| asset_server.load(format!("ZombieOGA/attack/__Zombie01_Attack_{:03}.png", i)))
        .collect();
    animations.insert(AnimationState::Attack, Animation::new(attack_frames, 0.1));

    // Load hurt animation (7 frames)
    let hurt_frames: Vec<Handle<Image>> = (0..7)
        .map(|i| asset_server.load(format!("ZombieOGA/hurt/__Zombie01_Hurt_{:03}.png", i)))
        .collect();
    animations.insert(AnimationState::Hurt, Animation::new(hurt_frames, 0.1));

    // Load dead animation (7 frames)
    let dead_frames: Vec<Handle<Image>> = (0..7)
        .map(|i| asset_server.load(format!("ZombieOGA/dead/__Zombie01_Dead_{:03}.png", i)))
        .collect();
    animations.insert(AnimationState::Dead, Animation::new(dead_frames, 0.1));

    animations
}

pub fn animate_sprites(time: Res<Time>, mut query: Query<(&mut AnimatedSprite, &mut Sprite)>) {
    for (mut animated, mut sprite) in &mut query {
        animated.timer.tick(time.delta());

        if animated.timer.just_finished() {
            let current_state = animated.current_state;

            if let Some(animation) = animated.animations.get_mut(&current_state) {
                // Advance to next frame
                animation.current_frame = (animation.current_frame + 1) % animation.frames.len();

                // Clone the handle to avoid borrow issues
                let frame_handle = animation.frames[animation.current_frame].clone();

                // Update sprite texture
                sprite.image = frame_handle;
            }
        }
    }
}
