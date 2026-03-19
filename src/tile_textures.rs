use std::collections::HashMap;
use bevy::prelude::*;

/// Loaded tile texture handles keyed by asset_id (e.g. "grass_01").
/// Populated when world assets finish downloading.
#[derive(Resource, Default)]
pub struct TileTextures {
    pub textures: HashMap<String, Handle<Image>>,
    pub loaded: bool,
}

impl TileTextures {
    pub fn clear(&mut self) {
        self.textures.clear();
        self.loaded = false;
    }
}
