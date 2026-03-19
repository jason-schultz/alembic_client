use bevy::prelude::*;

/// Commands fired by UI systems → consumed by network systems.
/// UI never touches NetworkConnection directly.
#[derive(Event, Debug)]
pub enum NetworkCommand {
    /// Connect to the given address and begin the handshake.
    Connect { addr: String },
    /// Disconnect cleanly.
    Disconnect,
}

/// Events fired by network systems → consumed by UI / game systems.
#[derive(Event, Debug, Clone)]
pub enum NetworkEvent {
    /// TCP connection established, handshake sent.
    Connecting,
    /// Auth succeeded — player is live on the server.
    Authenticated {
        player_id: String,
        session_id: String,
    },
    /// Auth failed with a reason code.
    AuthFailed { reason: u8 },
    /// Server closed the connection or an IO error occurred.
    Disconnected { reason: String },
    /// Server sent the list of available worlds after auth.
    WorldList { worlds: Vec<WorldInfo> },
    /// Zone info received after joining a world.
    ZoneInfo {
        zone_id: String,
        zone_name: String,
        width: u16,
        height: u16,
    },
    /// Room info received (player is inside a room).
    RoomInfo {
        room_id: String,
        room_name: String,
        width: u16,
        height: u16,
    },
    /// Authoritative spawn position from the server.
    SpawnPosition {
        zone_id: String,
        x: u16,
        y: u16,
        world_x: u32,
        world_y: u32,
        facing: u8,
    },
    PositionConfirmed {
        zone_id: String,
        x: u16,
        y: u16,
        world_x: u32,
        world_y: u32,
        facing: u8,
    },
    PositionCorrection {
        x: u16,
        y: u16,
        world_x: u32,
        world_y: u32,
        facing: u8,
    },
    /// Viewport tile data around the player's position.
    ViewportUpdate {
        center_x: u16,
        center_y: u16,
        tiles: Vec<TileData>,
    },
}

/// A single world entry from a WORLD_LIST packet.
#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub world_id: String,
    pub world_name: String,
    pub description: String,
    pub player_count: u16,
}

/// A single tile in a viewport update.
#[derive(Debug, Clone)]
pub struct TileData {
    pub x: u16,
    pub y: u16,
    pub asset_id: String,
    pub walkable: bool,
}
