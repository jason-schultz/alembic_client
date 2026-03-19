use super::events::{TileData, WorldInfo};

// ── Packet IDs (server → client) ───────────────────────────────────────────
pub const HANDSHAKE_RESPONSE: u16 = 0x0002;
pub const AUTH_SUCCESS: u16 = 0x0011;
pub const AUTH_FAILURE: u16 = 0x0012;
pub const HEARTBEAT: u16 = 0x0022;
pub const ZONE_INFO: u16 = 0x0200;
pub const ROOM_INFO: u16 = 0x0201;
pub const SPAWN_POSITION: u16 = 0x0202;
pub const VIEWPORT_UPDATE: u16 = 0x0101;
pub const POSITION_CONFIRM: u16 = 0x0203;
pub const POSITION_CORRECTION: u16 = 0x0204;
pub const WORLD_LIST: u16 = 0x0302;

// ── Packet IDs (client → server) ───────────────────────────────────────────
pub const HANDSHAKE_REQUEST: u16 = 0x0001;
pub const AUTH_REQUEST: u16 = 0x0010;
pub const HEARTBEAT_ACK: u16 = 0x0023;
pub const PLAYER_MOVE: u16 = 0x0100;
pub const JOIN_WORLD: u16 = 0x0300;
pub const LEAVE_WORLD: u16 = 0x0301;

pub const MAGIC: [u8; 4] = [0x41, 0x4C, 0x42, 0x43];
pub const VERSION: [u8; 3] = [0, 1, 0];

/// Every packet type the server can send us.
#[derive(Debug, Clone)]
pub enum ServerPacket {
    HandshakeResponse {
        challenge: [u8; 32],
    },
    AuthSuccess {
        session_id: String,
        player_id: String,
    },
    AuthFailure {
        reason: u8,
    },
    Disconnected {
        reason: String,
    },
    Heartbeat,
    WorldList {
        worlds: Vec<WorldInfo>,
    },
    ZoneInfo {
        zone_id: String,
        zone_name: String,
        width: u16,
        height: u16,
    },
    RoomInfo {
        room_id: String,
        room_name: String,
        width: u16,
        height: u16,
    },
    SpawnPosition {
        zone_id: String,
        x: u16,
        y: u16,
        world_x: u32,
        world_y: u32,
        facing: u8,
    },
    ViewportUpdate {
        center_x: u16,
        center_y: u16,
        tiles: Vec<TileData>,
    },
    PositionConfirm {
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
    Unknown {
        id: u16,
        payload: Vec<u8>,
    },
}

/// Try to parse one complete packet from the front of `buf`.
/// Returns `Some((packet, bytes_consumed))` or `None` if more data is needed.
pub fn try_parse_packet(buf: &[u8]) -> Option<(ServerPacket, usize)> {
    // Header: 4 magic + 3 version + 2 id + 4 length = 13 bytes
    if buf.len() < 13 {
        return None;
    }
    if &buf[0..4] != &MAGIC {
        return None;
    }
    let packet_id = u16::from_be_bytes([buf[7], buf[8]]);
    let length = u32::from_be_bytes([buf[9], buf[10], buf[11], buf[12]]) as usize;

    let total = 13 + length;
    if buf.len() < total {
        return None;
    }

    let payload = &buf[13..total];
    let packet = parse_payload(packet_id, payload);
    Some((packet, total))
}

fn parse_payload(id: u16, payload: &[u8]) -> ServerPacket {
    match id {
        HANDSHAKE_RESPONSE => parse_handshake_response(id, payload),
        AUTH_SUCCESS => parse_auth_success(id, payload),
        AUTH_FAILURE => {
            let reason = payload.first().copied().unwrap_or(0xFF);
            ServerPacket::AuthFailure { reason }
        }
        HEARTBEAT => ServerPacket::Heartbeat,
        WORLD_LIST => parse_world_list(id, payload),
        ZONE_INFO => parse_zone_info(id, payload),
        ROOM_INFO => parse_room_info(id, payload),
        SPAWN_POSITION => parse_spawn_position(id, payload),
        VIEWPORT_UPDATE => parse_viewport_update(id, payload),
        POSITION_CONFIRM => parse_position_confirm(id, payload),
        POSITION_CORRECTION => parse_position_correction(id, payload),
        _ => ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        },
    }
}

fn read_length_prefixed_string(payload: &[u8], offset: usize) -> Option<(String, usize)> {
    if payload.len() < offset + 2 {
        return None;
    }
    let len = u16::from_be_bytes([payload[offset], payload[offset + 1]]) as usize;
    let end = offset + 2 + len;
    if payload.len() < end {
        return None;
    }
    let s = String::from_utf8_lossy(&payload[offset + 2..end]).to_string();
    Some((s, end))
}

fn parse_handshake_response(id: u16, payload: &[u8]) -> ServerPacket {
    if payload.len() == 32 {
        let mut challenge = [0u8; 32];
        challenge.copy_from_slice(payload);
        ServerPacket::HandshakeResponse { challenge }
    } else {
        ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        }
    }
}

fn parse_auth_success(id: u16, payload: &[u8]) -> ServerPacket {
    let Some((session_id, cursor)) = read_length_prefixed_string(payload, 0) else {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    };
    let Some((player_id, _)) = read_length_prefixed_string(payload, cursor) else {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    };
    ServerPacket::AuthSuccess {
        session_id,
        player_id,
    }
}

fn parse_world_list(id: u16, payload: &[u8]) -> ServerPacket {
    if payload.len() < 2 {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    }
    let count = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    let mut cursor = 2;
    let mut worlds = Vec::with_capacity(count);

    for _ in 0..count {
        let Some((world_id, next)) = read_length_prefixed_string(payload, cursor) else {
            break;
        };
        cursor = next;

        let Some((world_name, next)) = read_length_prefixed_string(payload, cursor) else {
            break;
        };
        cursor = next;

        let Some((description, next)) = read_length_prefixed_string(payload, cursor) else {
            break;
        };
        cursor = next;

        if payload.len() < cursor + 2 {
            break;
        }
        let player_count = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]);
        cursor += 2;

        worlds.push(WorldInfo {
            world_id,
            world_name,
            description,
            player_count,
        });
    }

    ServerPacket::WorldList { worlds }
}

fn parse_zone_info(id: u16, payload: &[u8]) -> ServerPacket {
    let Some((zone_id, cursor)) = read_length_prefixed_string(payload, 0) else {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    };
    let Some((zone_name, cursor)) = read_length_prefixed_string(payload, cursor) else {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    };
    if payload.len() < cursor + 4 {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    }
    let width = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]);
    let height = u16::from_be_bytes([payload[cursor + 2], payload[cursor + 3]]);
    ServerPacket::ZoneInfo {
        zone_id,
        zone_name,
        width,
        height,
    }
}

fn parse_room_info(id: u16, payload: &[u8]) -> ServerPacket {
    let Some((room_id, cursor)) = read_length_prefixed_string(payload, 0) else {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    };
    let Some((room_name, cursor)) = read_length_prefixed_string(payload, cursor) else {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    };
    if payload.len() < cursor + 4 {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    }
    let width = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]);
    let height = u16::from_be_bytes([payload[cursor + 2], payload[cursor + 3]]);
    ServerPacket::RoomInfo {
        room_id,
        room_name,
        width,
        height,
    }
}

fn parse_spawn_position(id: u16, payload: &[u8]) -> ServerPacket {
    let Some((zone_id, cursor)) = read_length_prefixed_string(payload, 0) else {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    };
    // x::16, y::16, world_x::32, world_y::32, facing::8 = 13 bytes
    if payload.len() < cursor + 13 {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    }
    let x = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]);
    let y = u16::from_be_bytes([payload[cursor + 2], payload[cursor + 3]]);
    let world_x = u32::from_be_bytes([
        payload[cursor + 4],
        payload[cursor + 5],
        payload[cursor + 6],
        payload[cursor + 7],
    ]);
    let world_y = u32::from_be_bytes([
        payload[cursor + 8],
        payload[cursor + 9],
        payload[cursor + 10],
        payload[cursor + 11],
    ]);
    let facing = payload[cursor + 12];
    ServerPacket::SpawnPosition {
        zone_id,
        x,
        y,
        world_x,
        world_y,
        facing,
    }
}

fn parse_viewport_update(id: u16, payload: &[u8]) -> ServerPacket {
    // center_x::16, center_y::16, tile_count::32 = 8 bytes header
    if payload.len() < 8 {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    }
    let center_x = u16::from_be_bytes([payload[0], payload[1]]);
    let center_y = u16::from_be_bytes([payload[2], payload[3]]);
    let tile_count = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]) as usize;

    let mut tiles = Vec::with_capacity(tile_count);
    let mut cursor = 8;

    for _ in 0..tile_count {
        if payload.len() < cursor + 4 {
            break;
        }
        let x = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]);
        let y = u16::from_be_bytes([payload[cursor + 2], payload[cursor + 3]]);
        cursor += 4;

        let Some((asset_id, next)) = read_length_prefixed_string(payload, cursor) else {
            break;
        };
        cursor = next;

        if payload.len() < cursor + 1 {
            break;
        }
        let walkable = payload[cursor] == 1;
        cursor += 1;

        tiles.push(TileData {
            x,
            y,
            asset_id,
            walkable,
        });
    }

    ServerPacket::ViewportUpdate {
        center_x,
        center_y,
        tiles,
    }
}

fn parse_position_confirm(id: u16, payload: &[u8]) -> ServerPacket {
    let Some((zone_id, cursor)) = read_length_prefixed_string(payload, 0) else {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    };
    if payload.len() < cursor + 13 {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    }
    let x = u16::from_be_bytes([payload[cursor], payload[cursor + 1]]);
    let y = u16::from_be_bytes([payload[cursor + 2], payload[cursor + 3]]);
    let world_x = u32::from_be_bytes([
        payload[cursor + 4],
        payload[cursor + 5],
        payload[cursor + 6],
        payload[cursor + 7],
    ]);
    let world_y = u32::from_be_bytes([
        payload[cursor + 8],
        payload[cursor + 9],
        payload[cursor + 10],
        payload[cursor + 11],
    ]);
    let facing = payload[cursor + 12];

    ServerPacket::PositionConfirm {
        zone_id,
        x,
        y,
        world_x,
        world_y,
        facing,
    }
}

fn parse_position_correction(id: u16, payload: &[u8]) -> ServerPacket {
    if payload.len() < 13 {
        return ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        };
    }
    let x = u16::from_be_bytes([payload[0], payload[1]]);
    let y = u16::from_be_bytes([payload[2], payload[3]]);
    let world_x = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let world_y = u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]);
    let facing = payload[12];

    ServerPacket::PositionCorrection {
        x,
        y,
        world_x,
        world_y,
        facing,
    }
}
