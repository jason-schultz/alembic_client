use std::io::Read;

use bevy::prelude::*;

use super::connection::{ConnectionPhase, NetworkConnection};
use super::events::{NetworkCommand, NetworkEvent};
use super::packets::{ServerPacket, try_parse_packet};
use crate::auth::AuthToken;
use crate::network::PacketQueue;

/// Reads `NetworkCommand` events fired by UI and acts on them.
/// This is the only place that calls `network.connect()`.
pub fn handle_network_commands(
    mut commands_reader: EventReader<NetworkCommand>,
    network: Res<NetworkConnection>,
    mut events: EventWriter<NetworkEvent>,
) {
    for command in commands_reader.read() {
        match command {
            NetworkCommand::Connect { addr } => {
                if !network.is_disconnected() {
                    warn!("Connect requested but already connecting/connected — ignoring");
                    continue;
                }

                info!("Connecting to {addr}...");
                events.send(NetworkEvent::Connecting);

                let net = network.clone();
                let addr = addr.clone();

                std::thread::spawn(move || match net.connect(&addr) {
                    Ok(()) => {
                        if let Err(e) = net.send_handshake_request("alembic-client") {
                            error!("Handshake failed: {e}");
                            net.set_phase(ConnectionPhase::Failed(e.to_string()));
                        }
                    }
                    Err(e) => {
                        error!("TCP connect failed: {e}");
                        net.set_phase(ConnectionPhase::Failed(e.to_string()));
                    }
                });
            }

            NetworkCommand::Disconnect => {
                info!("Disconnecting...");
                *network.stream.lock().unwrap() = None;
                network.set_phase(ConnectionPhase::Disconnected);
                events.send(NetworkEvent::Disconnected {
                    reason: "User disconnected".into(),
                });
            }
        }
    }
}

/// Reads raw bytes from the TCP socket into the buffer every frame,
/// then parses as many complete packets as possible and pushes them
/// onto a temporary queue for `process_packets` to handle.
pub fn poll_network(network: Res<NetworkConnection>, mut queue: ResMut<PacketQueue>) {
    queue.0.clear();

    // 1. Read bytes from socket
    {
        let mut stream_guard = network.stream.lock().unwrap();
        if let Some(stream) = stream_guard.as_mut() {
            let mut raw_buf = [0u8; 4096];
            match stream.read(&mut raw_buf) {
                Ok(0) => {
                    drop(stream_guard);
                    *network.stream.lock().unwrap() = None;
                    network.set_phase(ConnectionPhase::Failed("Server closed connection".into()));
                    queue.0.push(ServerPacket::Disconnected {
                        reason: "Server closed connection".into(),
                    });
                    return;
                }
                Ok(n) => {
                    network
                        .buffer
                        .lock()
                        .unwrap()
                        .extend_from_slice(&raw_buf[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    drop(stream_guard);
                    *network.stream.lock().unwrap() = None;
                    network.set_phase(ConnectionPhase::Failed(e.to_string()));
                    return;
                }
            }
        } else {
            return;
        }
    }

    // 2. Parse complete packets from buffer
    loop {
        let consumed = {
            let buf = network.buffer.lock().unwrap();
            match try_parse_packet(&buf) {
                Some((packet, consumed)) => {
                    queue.0.push(packet);
                    consumed
                }
                None => break,
            }
        };
        network.buffer.lock().unwrap().drain(..consumed);
    }
}

/// Converts raw `ServerPacket`s into `NetworkEvent`s.
/// Handles phase transitions and fires appropriate events for the rest of the
/// app to consume — UI, game world, etc.
pub fn process_packets(
    network: Res<NetworkConnection>,
    auth: Res<AuthToken>,
    queue: Res<PacketQueue>,
    mut events: EventWriter<NetworkEvent>,
) {
    for packet in queue.0.iter() {
        match packet {
            ServerPacket::HandshakeResponse { challenge } => {
                info!("Challenge received, sending auth...");
                if let Err(e) = network.send_auth_request(&auth.token, challenge) {
                    error!("Failed to send auth request: {e}");
                }
            }

            ServerPacket::AuthSuccess {
                session_id,
                player_id,
            } => {
                info!("Authenticated! player_id={player_id}");
                network.set_phase(ConnectionPhase::Active);
                events.send(NetworkEvent::Authenticated {
                    player_id: player_id.clone(),
                    session_id: session_id.clone(),
                });
            }

            ServerPacket::AuthFailure { reason } => {
                error!("Auth failed: reason code {reason:#04x}");
                network.set_phase(ConnectionPhase::Failed(format!(
                    "Auth failed: code {reason}"
                )));
                events.send(NetworkEvent::AuthFailed { reason: *reason });
            }

            ServerPacket::Disconnected { reason } => {
                info!("Disconnected: {reason}");
                network.set_phase(ConnectionPhase::Disconnected);
                events.send(NetworkEvent::Disconnected {
                    reason: reason.clone(),
                });
            }

            ServerPacket::Heartbeat => {
                if let Err(e) = network.send_heartbeat_ack() {
                    error!("Failed to send heartbeat ack: {e}");
                }
            }

            ServerPacket::ZoneInfo {
                zone_id,
                zone_name,
                width,
                height,
            } => {
                info!("Zone info received: {zone_id} ({width}x{height})");
                events.send(NetworkEvent::ZoneInfo {
                    zone_id: zone_id.clone(),
                    zone_name: zone_name.clone(),
                    width: *width,
                    height: *height,
                });
            }

            ServerPacket::RoomInfo {
                room_id,
                room_name,
                width,
                height,
            } => {
                info!("Room info received: {room_id} ({width}x{height})");
                events.send(NetworkEvent::RoomInfo {
                    room_id: room_id.clone(),
                    room_name: room_name.clone(),
                    width: *width,
                    height: *height,
                });
            }

            ServerPacket::SpawnPosition {
                zone_id,
                x,
                y,
                world_x,
                world_y,
                facing,
            } => {
                info!("Spawn position received: zone={zone_id} x={x} y={y}");
                events.send(NetworkEvent::SpawnPosition {
                    zone_id: zone_id.clone(),
                    x: *x,
                    y: *y,
                    world_x: *world_x,
                    world_y: *world_y,
                    facing: *facing,
                });
            }

            ServerPacket::PositionConfirm {
                zone_id,
                x,
                y,
                world_x,
                world_y,
                facing,
            } => {
                info!("Position confirmed: zone={zone_id} x={x} y={y}");
                events.send(NetworkEvent::PositionConfirmed {
                    zone_id: zone_id.clone(),
                    x: *x,
                    y: *y,
                    world_x: *world_x,
                    world_y: *world_y,
                    facing: *facing,
                });
            }

            ServerPacket::PositionCorrection {
                x,
                y,
                world_x,
                world_y,
                facing,
            } => {
                info!("Position correction: x={x} y={y}");
                events.send(NetworkEvent::PositionCorrection {
                    x: *x,
                    y: *y,
                    world_x: *world_x,
                    world_y: *world_y,
                    facing: *facing,
                });
            }

            ServerPacket::ViewportUpdate {
                center_x,
                center_y,
                tiles,
            } => {
                info!(
                    "Viewport update: {} tiles around ({center_x},{center_y})",
                    tiles.len()
                );
                events.send(NetworkEvent::ViewportUpdate {
                    center_x: *center_x,
                    center_y: *center_y,
                    tiles: tiles.clone(),
                });
            }

            ServerPacket::WorldList { worlds } => {
                info!("World list received: {} worlds", worlds.len());
                events.send(NetworkEvent::WorldList {
                    worlds: worlds.clone(),
                });
            }

            ServerPacket::Unknown { id, .. } => {
                warn!("Unknown packet id: {id:#06x}");
            }
        }
    }
}
