use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::asset_cache::{AnimationDef, TilesetEntry, SpritesheetEntry};

/// A character sprite available for selection, with its local cached path.
#[derive(Debug, Clone)]
pub struct AvailableSprite {
    pub id: String,
    pub local_path: PathBuf,
    /// Bevy handle — loaded once the asset is ready
    pub handle: Option<Handle<Image>>,
    pub cell_width: u32,
    pub cell_height: u32,
    pub animations: HashMap<String, AnimationDef>,
}

/// Holds all character sprites downloaded from the current server.
/// Populated during the Connecting state after auth succeeds.
#[derive(Resource, Default)]
pub struct AvailableSprites {
    pub sprites: Vec<AvailableSprite>,
    pub loaded: bool,
    pub loading: bool,
    pub error: Option<String>,
}

impl AvailableSprites {
    pub fn clear(&mut self) {
        self.sprites.clear();
        self.loaded = false;
        self.loading = false;
        self.error = None;
    }
}

/// Fired when sprite downloading completes (from background thread).
#[derive(Event)]
pub struct SpritesReady {
    pub sprites: Vec<(SpritesheetEntry, PathBuf)>,
    pub tiles: Vec<(TilesetEntry, PathBuf)>,
}

/// Fired when sprite downloading fails.
#[derive(Event)]
pub struct SpritesFailed {
    pub reason: String,
}
