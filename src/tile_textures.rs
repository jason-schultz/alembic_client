use std::collections::HashMap;
use bevy::prelude::*;

use crate::asset_cache::TilesetEntry;

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

#[derive(Default, Resource)]
pub struct TileAtlases {
    pub atlases: HashMap<String, (Handle<TextureAtlasLayout>, Handle<Image>)>,
    pub label_lookup: HashMap<String, (String, usize)>,
    pub loaded: bool,
}

impl TileAtlases {

}