use bevy::prelude::*;
use std::collections::HashMap;

use crate::asset_cache::AnimationDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    IdleSouth,
    IdleNorth,
    IdleEast,
    IdleWest,
    Walk,
    WalkNorth,
    WalkSouth,
    WalkEast,
    WalkWest,
    Attack,
    AttackSouth,
    AttackNorth,
    AttackEast,
    AttackWest,
    Hurt,
    Dead,
}

/// A single animation — either individual frame files or rects within a sprite sheet.
#[derive(Clone)]
pub struct Animation {
    pub frames: AnimationFrames,
    pub current_frame: usize,
    pub frame_time: f32,
    pub flip_x: bool,
}

#[derive(Clone)]
pub enum AnimationFrames {
    /// Individual image handles (zombie-style, one PNG per frame)
    Individual(Vec<Handle<Image>>),
    /// Sprite sheet — one image, list of UV rects per frame
    Sheet {
        image: Handle<Image>,
        rects: Vec<bevy::math::Rect>,
    },
}

impl Animation {
    pub fn from_frames(frames: Vec<Handle<Image>>, frame_time: f32) -> Self {
        Self {
            frames: AnimationFrames::Individual(frames),
            current_frame: 0,
            frame_time,
            flip_x: false,
        }
    }

    pub fn from_sheet(image: Handle<Image>, rects: Vec<bevy::math::Rect>, frame_time: f32) -> Self {
        Self {
            frames: AnimationFrames::Sheet { image, rects },
            current_frame: 0,
            frame_time,
            flip_x: false,
        }
    }

    pub fn frame_count(&self) -> usize {
        match &self.frames {
            AnimationFrames::Individual(v) => v.len(),
            AnimationFrames::Sheet { rects, .. } => rects.len(),
        }
    }

    pub fn first_image(&self) -> Option<Handle<Image>> {
        match &self.frames {
            AnimationFrames::Individual(v) => v.first().cloned(),
            AnimationFrames::Sheet { image, .. } => Some(image.clone()),
        }
    }

    pub fn first_rect(&self) -> Option<bevy::math::Rect> {
        match &self.frames {
            AnimationFrames::Individual(_) => None,
            AnimationFrames::Sheet { rects, .. } => rects.first().cloned(),
        }
    }
}

#[derive(Component)]
pub struct AnimatedSprite {
    pub current_state: AnimationState,
    pub animations: HashMap<AnimationState, Animation>,
    pub timer: Timer,
}

// ── Sprite sheet animations (paper doll characters) ──────────────────────────

/// Maps a manifest animation key to its AnimationState variant.
fn key_to_state(key: &str) -> Option<AnimationState> {
    match key {
        "idle_south"   => Some(AnimationState::IdleSouth),
        "idle_north"   => Some(AnimationState::IdleNorth),
        "idle_east"    => Some(AnimationState::IdleEast),
        "idle_west"    => Some(AnimationState::IdleWest),
        "walk_south"   => Some(AnimationState::WalkSouth),
        "walk_north"   => Some(AnimationState::WalkNorth),
        "walk_east"    => Some(AnimationState::WalkEast),
        "walk_west"    => Some(AnimationState::WalkWest),
        "attack_south" => Some(AnimationState::AttackSouth),
        "attack_north" => Some(AnimationState::AttackNorth),
        "attack_east"  => Some(AnimationState::AttackEast),
        "attack_west"  => Some(AnimationState::AttackWest),
        "dying"        => Some(AnimationState::Dead),
        _ => None,
    }
}

/// Builds the animation map from manifest-provided AnimationDefs.
pub fn load_animations_from_manifest(
    image: Handle<Image>,
    cell_width: u32,
    cell_height: u32,
    defs: &HashMap<String, AnimationDef>,
) -> HashMap<AnimationState, Animation> {
    let fw = cell_width as f32;
    let fh = cell_height as f32;

    let mut animations = HashMap::new();

    for (key, def) in defs {
        let Some(state) = key_to_state(key) else { continue };

        let rects: Vec<bevy::math::Rect> = (0..def.frames)
            .map(|i| {
                let col = def.start_col + i;
                bevy::math::Rect {
                    min: Vec2::new(col as f32 * fw, def.row as f32 * fh),
                    max: Vec2::new((col + 1) as f32 * fw, (def.row + 1) as f32 * fh),
                }
            })
            .collect();

        let mut anim = Animation::from_sheet(image.clone(), rects, def.frame_time);
        anim.flip_x = def.flip_x;
        animations.insert(state, anim);
    }

    // Alias: Idle → IdleSouth so the default state always resolves
    if let Some(idle_south) = animations.get(&AnimationState::IdleSouth).cloned() {
        animations.insert(AnimationState::Idle, idle_south);
    }

    animations
}

// ── Animation system ──────────────────────────────────────────────────────────

pub fn animate_sprites(time: Res<Time>, mut query: Query<(&mut AnimatedSprite, &mut Sprite)>) {
    for (mut animated, mut sprite) in &mut query {
        animated.timer.tick(time.delta());

        if !animated.timer.just_finished() {
            continue;
        }

        let current_state = animated.current_state;

        if let Some(animation) = animated.animations.get_mut(&current_state) {
            animation.current_frame = (animation.current_frame + 1) % animation.frame_count();

            sprite.flip_x = animation.flip_x;
            match &animation.frames {
                AnimationFrames::Individual(frames) => {
                    sprite.image = frames[animation.current_frame].clone();
                    sprite.rect = None;
                }
                AnimationFrames::Sheet { image, rects } => {
                    sprite.image = image.clone();
                    sprite.rect = Some(rects[animation.current_frame]);
                }
            }
        }
    }
}
