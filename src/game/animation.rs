use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walk,
    WalkNorth,
    WalkSouth,
    WalkEast,
    WalkWest,
    Attack,
    Hurt,
    Dead,
}

/// A single animation — either individual frame files or rects within a sprite sheet.
#[derive(Clone)]
pub struct Animation {
    pub frames: AnimationFrames,
    pub current_frame: usize,
    pub frame_time: f32,
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
        }
    }

    pub fn from_sheet(image: Handle<Image>, rects: Vec<bevy::math::Rect>, frame_time: f32) -> Self {
        Self {
            frames: AnimationFrames::Sheet { image, rects },
            current_frame: 0,
            frame_time,
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
//
// Sheet layout for the character PNGs (144x256, 3 cols x 4 rows, 48x64 each):
//
//   Row 0 (y=0):   Front idle, Front walk-1, Front walk-2
//   Row 1 (y=64):  Side  idle, Side  walk-1, Side  walk-2
//   Row 2 (y=128): Back  idle, Back  walk-1, Back  walk-2
//   Row 3 (y=192): Side2 idle, Side2 walk-1, Side2 walk-2
//
// For now we map:
//   Idle  → front idle (col 0, row 0)
//   Walk  → front walk cycle (cols 0-2, row 0)
//   Other states fall back to idle

pub fn load_sheet_animations(image: Handle<Image>) -> HashMap<AnimationState, Animation> {
    let mut animations = HashMap::new();
    let fw = 48.0_f32;
    let fh = 64.0_f32;

    let rect = |col: u32, row: u32| bevy::math::Rect {
        min: Vec2::new(col as f32 * fw, row as f32 * fh),
        max: Vec2::new((col + 1) as f32 * fw, (row + 1) as f32 * fh),
    };

    // Row 0: South
    animations.insert(
        AnimationState::WalkSouth,
        Animation::from_sheet(
            image.clone(),
            vec![rect(0, 0), rect(1, 0), rect(2, 0)],
            0.18,
        ),
    );
    // Row 1: West
    animations.insert(
        AnimationState::WalkWest,
        Animation::from_sheet(
            image.clone(),
            vec![rect(0, 1), rect(1, 1), rect(2, 1)],
            0.18,
        ),
    );
    // Row 2: North
    animations.insert(
        AnimationState::WalkNorth,
        Animation::from_sheet(
            image.clone(),
            vec![rect(0, 2), rect(1, 2), rect(2, 2)],
            0.18,
        ),
    );
    // Row 3: East
    animations.insert(
        AnimationState::WalkEast,
        Animation::from_sheet(
            image.clone(),
            vec![rect(0, 3), rect(1, 3), rect(2, 3)],
            0.18,
        ),
    );

    // Idle — south facing col 0
    animations.insert(
        AnimationState::Idle,
        Animation::from_sheet(image.clone(), vec![rect(0, 0)], 0.5),
    );

    for state in [
        AnimationState::Attack,
        AnimationState::Hurt,
        AnimationState::Dead,
    ] {
        animations.insert(
            state,
            Animation::from_sheet(image.clone(), vec![rect(0, 0)], 0.5),
        );
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
