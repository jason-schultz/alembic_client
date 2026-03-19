use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

const MAX_ASSET_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];

// ── Manifest types ────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct WorldManifest {
    pub tiles: Vec<TileAssetEntry>,
    pub sprites: SpriteManifest,
}

#[derive(Debug, serde::Deserialize)]
pub struct SpriteManifest {
    pub characters: Vec<CharacterSpriteEntry>,
    #[serde(default)]
    pub npcs: Vec<AssetEntry>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TileAssetEntry {
    pub id: String,
    pub file: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CharacterSpriteEntry {
    pub id: String,
    pub filename: String,
    pub url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AssetEntry {
    pub id: String,
    pub file: String,
}

// ── Sanitization ──────────────────────────────────────────────────────────────

/// Validates and extracts a safe basename from a server-provided filename.
/// Rejects path traversal, hidden files, null bytes, and non-image extensions.
fn sanitize_filename(filename: &str) -> Option<String> {
    if filename.is_empty() || filename.contains('\0') {
        return None;
    }
    let name = std::path::Path::new(filename).file_name()?.to_str()?;
    if name.starts_with('.') {
        return None;
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return None;
    }
    let lower = name.to_lowercase();
    if !lower.ends_with(".png") && !lower.ends_with(".jpg") && !lower.ends_with(".jpeg") {
        return None;
    }
    if name.len() > 128 {
        return None;
    }
    Some(name.to_string())
}

/// Validates a path component (e.g. world_id) for use in a filesystem path.
fn sanitize_path_component(s: &str) -> Option<&str> {
    if s.is_empty() || s.len() > 64 {
        return None;
    }
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Some(s)
    } else {
        None
    }
}

/// Validates a relative server asset path (e.g. "tiles/grass_01.png") for safe
/// use in URL construction. Rejects traversal, absolute paths, and unsafe chars.
fn sanitize_asset_path(path: &str) -> Option<&str> {
    if path.is_empty() || path.len() > 256 || path.contains('\0') {
        return None;
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return None;
    }
    for component in path.split('/') {
        if component == ".." || component == "." || component.is_empty() {
            return None;
        }
    }
    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.')
    {
        return None;
    }
    let lower = path.to_lowercase();
    if !lower.ends_with(".png") && !lower.ends_with(".jpg") && !lower.ends_with(".jpeg") {
        return None;
    }
    Some(path)
}

fn validate_image_magic(bytes: &[u8], filename: &str) -> bool {
    let lower = filename.to_lowercase();
    if lower.ends_with(".png") {
        bytes.starts_with(PNG_MAGIC)
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        bytes.starts_with(JPEG_MAGIC)
    } else {
        false
    }
}

// ── Cache directory helpers ───────────────────────────────────────────────────

fn safe_addr(server_addr: &str) -> String {
    server_addr
        .replace(':', "_")
        .replace('/', "_")
        .replace('\\', "_")
}

fn safe_world(world_id: &str) -> String {
    world_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Cache dir for character sprites.
/// e.g. ~/.cache/alembic/servers/127.0.0.1_7777/worlds/main_story/sprites/characters/
pub fn cache_dir_for_server(server_addr: &str, world_id: &str) -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".cache"));
    base.join("alembic")
        .join("servers")
        .join(safe_addr(server_addr))
        .join("worlds")
        .join(safe_world(world_id))
        .join("sprites")
        .join("characters")
}

/// Cache dir for tile assets.
/// e.g. ~/.cache/alembic/servers/127.0.0.1_7777/worlds/main_story/tiles/
pub fn tile_cache_dir_for_server(server_addr: &str, world_id: &str) -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".cache"));
    base.join("alembic")
        .join("servers")
        .join(safe_addr(server_addr))
        .join("worlds")
        .join(safe_world(world_id))
        .join("tiles")
}

// ── HTTP fetch ────────────────────────────────────────────────────────────────

/// Fetches the unified world manifest.
/// Server must implement: GET /worlds/{world_id}/manifest
pub fn fetch_world_manifest(
    base_url: &str,
    world_id: &str,
) -> Result<WorldManifest, Box<dyn std::error::Error>> {
    let url = format!("{}/worlds/{}/manifest", base_url, world_id);
    let response = ureq::get(&url).call()?;
    let manifest: WorldManifest = response.into_json()?;
    Ok(manifest)
}

// ── Shared download helper ────────────────────────────────────────────────────

fn download_asset(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = ureq::get(url).call()?;
    let mut bytes: Vec<u8> = Vec::new();
    response
        .into_reader()
        .take((MAX_ASSET_SIZE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_ASSET_SIZE_BYTES {
        return Err(format!(
            "Download from '{}' exceeds maximum allowed size ({} bytes)",
            url, MAX_ASSET_SIZE_BYTES
        )
        .into());
    }
    Ok(bytes)
}

// ── Character sprites ─────────────────────────────────────────────────────────

/// Downloads a single character sprite if not already cached.
pub fn ensure_sprite_cached(
    base_url: &str,
    server_addr: &str,
    world_id: &str,
    entry: &CharacterSpriteEntry,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    sanitize_path_component(world_id)
        .ok_or_else(|| format!("Rejected unsafe world_id: '{}'", world_id))?;

    let safe_filename = sanitize_filename(&entry.filename)
        .ok_or_else(|| format!("Rejected unsafe sprite filename: '{}'", entry.filename))?;

    let cache_dir = cache_dir_for_server(server_addr, world_id);
    let local_path = cache_dir.join(&safe_filename);

    if local_path.exists() {
        return Ok(local_path);
    }

    fs::create_dir_all(&cache_dir)?;

    // url is an absolute path: /worlds/main_story/assets/sprites/characters/Male_White.png
    let url = format!("{}{}", base_url, entry.url);
    let bytes = download_asset(&url)?;

    if !validate_image_magic(&bytes, &safe_filename) {
        return Err(
            format!("Sprite '{}' failed format validation", safe_filename).into(),
        );
    }

    fs::File::create(&local_path)?.write_all(&bytes)?;
    Ok(local_path)
}

/// Downloads all character sprites that aren't already cached.
pub fn sync_sprite_cache(
    base_url: &str,
    server_addr: &str,
    world_id: &str,
    entries: &[CharacterSpriteEntry],
) -> Vec<(String, PathBuf)> {
    let mut results = Vec::new();
    for entry in entries {
        match ensure_sprite_cached(base_url, server_addr, world_id, entry) {
            Ok(path) => results.push((entry.id.clone(), path)),
            Err(e) => eprintln!("Failed to cache sprite {}: {e}", entry.id),
        }
    }
    results
}

// ── Tile assets ───────────────────────────────────────────────────────────────

/// Downloads a single tile asset if not already cached.
pub fn ensure_tile_cached(
    base_url: &str,
    server_addr: &str,
    world_id: &str,
    entry: &TileAssetEntry,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    sanitize_path_component(world_id)
        .ok_or_else(|| format!("Rejected unsafe world_id: '{}'", world_id))?;

    // Validate the relative path for URL construction before using it
    sanitize_asset_path(&entry.file)
        .ok_or_else(|| format!("Rejected unsafe tile path: '{}'", entry.file))?;

    // Store only the basename locally — tile_cache_dir already provides the directory
    let safe_filename = sanitize_filename(&entry.file)
        .ok_or_else(|| format!("Rejected unsafe tile filename: '{}'", entry.file))?;

    let cache_dir = tile_cache_dir_for_server(server_addr, world_id);
    let local_path = cache_dir.join(&safe_filename);

    if local_path.exists() {
        return Ok(local_path);
    }

    fs::create_dir_all(&cache_dir)?;

    // file is a relative path: "tiles/grass_01.png"
    let url = format!("{}/worlds/{}/assets/{}", base_url, world_id, entry.file);
    let bytes = download_asset(&url)?;

    if !validate_image_magic(&bytes, &safe_filename) {
        return Err(
            format!("Tile '{}' failed format validation", safe_filename).into(),
        );
    }

    fs::File::create(&local_path)?.write_all(&bytes)?;
    Ok(local_path)
}

/// Downloads all tile assets that aren't already cached.
pub fn sync_tile_cache(
    base_url: &str,
    server_addr: &str,
    world_id: &str,
    entries: &[TileAssetEntry],
) -> Vec<(String, PathBuf)> {
    let mut results = Vec::new();
    for entry in entries {
        match ensure_tile_cached(base_url, server_addr, world_id, entry) {
            Ok(path) => results.push((entry.id.clone(), path)),
            Err(e) => eprintln!("Failed to cache tile {}: {e}", entry.id),
        }
    }
    results
}
