use bevy::prelude::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

// ── Packet IDs ─────────────────────────────────────────────────────────────
const HANDSHAKE_REQUEST: u16 = 0x0001;
const HANDSHAKE_RESPONSE: u16 = 0x0002;
const AUTH_REQUEST: u16 = 0x0010;
const AUTH_SUCCESS: u16 = 0x0011;
const AUTH_FAILURE: u16 = 0x0012;
const HEARTBEAT: u16 = 0x0022;
const HEARTBEAT_ACK: u16 = 0x0023;

const MAGIC: [u8; 4] = [0x41, 0x4C, 0x42, 0x43];
// Match your mix.exs version
const VERSION: [u8; 3] = [0, 1, 0];

// ── Handshake state machine ─────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionPhase {
    /// TCP connected, handshake not yet sent
    Connected,
    /// Sent handshake_request, waiting for challenge
    AwaitingChallenge,
    /// Received challenge, sent auth_request, waiting for result
    Authenticating,
    /// Fully authenticated and active
    Active,
    /// Auth failed or disconnected
    Failed(String),
}

// ── Incoming server packets ─────────────────────────────────────────────────
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
    Heartbeat,
    Unknown {
        id: u16,
        payload: Vec<u8>,
    },
}

#[derive(Resource, Clone)]
pub struct NetworkConnection {
    pub stream: Arc<Mutex<Option<TcpStream>>>,
    pub incoming: Arc<Mutex<Vec<ServerPacket>>>,
    pub phase: Arc<Mutex<ConnectionPhase>>,
    /// Read buffer for partial packets
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Default for NetworkConnection {
    fn default() -> Self {
        Self {
            stream: Arc::new(Mutex::new(None)),
            incoming: Arc::new(Mutex::new(Vec::new())),
            phase: Arc::new(Mutex::new(ConnectionPhase::Connected)),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl NetworkConnection {
    pub fn connect(&self, addr: &str) -> Result<(), std::io::Error> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        *self.stream.lock().unwrap() = Some(stream);
        *self.phase.lock().unwrap() = ConnectionPhase::Connected;
        Ok(())
    }

    pub fn phase(&self) -> ConnectionPhase {
        self.phase.lock().unwrap().clone()
    }

    pub fn is_active(&self) -> bool {
        *self.phase.lock().unwrap() == ConnectionPhase::Active
    }

    // ── Packet builders ───────────────────────────────────────────────────

    /// [ALBC][ver][0x0001][len=client_id_len+2+2][u16:id_len][id bytes][u16:version]
    pub fn send_handshake_request(&self, client_id: &str) -> Result<(), std::io::Error> {
        let id_bytes = client_id.as_bytes();
        let mut payload = Vec::new();
        payload.extend_from_slice(&(id_bytes.len() as u16).to_be_bytes());
        payload.extend_from_slice(id_bytes);
        // protocol version as u16 — match your server expectation
        payload.extend_from_slice(&0x0100u16.to_be_bytes());

        self.send_raw(HANDSHAKE_REQUEST, &payload)?;
        *self.phase.lock().unwrap() = ConnectionPhase::AwaitingChallenge;
        Ok(())
    }

    /// [ALBC][ver][0x0010][len][u16:token_len][token bytes][32 bytes: hmac]
    pub fn send_auth_request(
        &self,
        token: &str,
        challenge: &[u8; 32],
    ) -> Result<(), std::io::Error> {
        let token_bytes = token.as_bytes();
        let hmac = compute_hmac(token_bytes, challenge);

        let mut payload = Vec::new();
        payload.extend_from_slice(&(token_bytes.len() as u16).to_be_bytes());
        payload.extend_from_slice(token_bytes);
        payload.extend_from_slice(&hmac);

        self.send_raw(AUTH_REQUEST, &payload)?;
        *self.phase.lock().unwrap() = ConnectionPhase::Authenticating;
        Ok(())
    }

    /// Heartbeat ack — sent in response to server heartbeat
    pub fn send_heartbeat_ack(&self) -> Result<(), std::io::Error> {
        self.send_raw(HEARTBEAT_ACK, &[])
    }

    fn send_raw(&self, packet_id: u16, payload: &[u8]) -> Result<(), std::io::Error> {
        let mut packet = Vec::with_capacity(13 + payload.len());
        packet.extend_from_slice(&MAGIC);
        packet.extend_from_slice(&VERSION);
        packet.extend_from_slice(&packet_id.to_be_bytes());
        packet.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        packet.extend_from_slice(payload);

        if let Some(stream) = self.stream.lock().unwrap().as_mut() {
            stream.write_all(&packet)?;
        }
        Ok(())
    }
}

// ── HMAC-SHA256(key=token, msg=challenge) ───────────────────────────────────
fn compute_hmac(token: &[u8], challenge: &[u8; 32]) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(token).expect("HMAC accepts any key length");
    mac.update(challenge);
    mac.finalize().into_bytes().into()
}

// ── Frame parser ────────────────────────────────────────────────────────────
fn try_parse_packet(buf: &[u8]) -> Option<(ServerPacket, usize)> {
    // Minimum header = 4 magic + 3 version + 2 id + 4 length = 13 bytes
    if buf.len() < 13 {
        return None;
    }
    if &buf[0..4] != &MAGIC {
        return None;
    }
    // bytes 4-6: version (ignored on client side)
    let packet_id = u16::from_be_bytes([buf[7], buf[8]]);
    let length = u32::from_be_bytes([buf[9], buf[10], buf[11], buf[12]]) as usize;

    let total = 13 + length;
    if buf.len() < total {
        return None; // wait for more data
    }

    let payload = &buf[13..total];
    let packet = parse_payload(packet_id, payload);
    Some((packet, total))
}

fn parse_payload(id: u16, payload: &[u8]) -> ServerPacket {
    match id {
        HANDSHAKE_RESPONSE => {
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
        AUTH_SUCCESS => {
            // [u16: session_len][session][u16: player_len][player]
            if payload.len() >= 2 {
                let session_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
                let session_end = 2 + session_len;
                if payload.len() >= session_end + 2 {
                    let session_id = String::from_utf8_lossy(&payload[2..session_end]).to_string();
                    let player_len =
                        u16::from_be_bytes([payload[session_end], payload[session_end + 1]])
                            as usize;
                    let player_end = session_end + 2 + player_len;
                    if payload.len() >= player_end {
                        let player_id =
                            String::from_utf8_lossy(&payload[session_end + 2..player_end])
                                .to_string();
                        return ServerPacket::AuthSuccess {
                            session_id,
                            player_id,
                        };
                    }
                }
            }
            ServerPacket::Unknown {
                id,
                payload: payload.to_vec(),
            }
        }
        AUTH_FAILURE => {
            let reason = payload.first().copied().unwrap_or(0xFF);
            ServerPacket::AuthFailure { reason }
        }
        HEARTBEAT => ServerPacket::Heartbeat,
        _ => ServerPacket::Unknown {
            id,
            payload: payload.to_vec(),
        },
    }
}

// ── Bevy system: runs every frame ───────────────────────────────────────────
pub fn poll_network(network: Res<NetworkConnection>) {
    let mut raw_buf = [0u8; 4096];

    // 1. Read bytes from socket into our persistent buffer
    {
        let mut stream_guard = network.stream.lock().unwrap();
        if let Some(stream) = stream_guard.as_mut() {
            match stream.read(&mut raw_buf) {
                Ok(0) => {
                    *stream_guard = None;
                    *network.phase.lock().unwrap() =
                        ConnectionPhase::Failed("Server closed connection".into());
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
                    *stream_guard = None;
                    *network.phase.lock().unwrap() = ConnectionPhase::Failed(e.to_string());
                    return;
                }
            }
        } else {
            return;
        }
    }

    // 2. Parse as many complete packets as possible
    loop {
        let consumed = {
            let buf = network.buffer.lock().unwrap();
            match try_parse_packet(&buf) {
                Some((packet, consumed)) => {
                    // Handle phase transitions inline
                    match &packet {
                        ServerPacket::AuthSuccess { .. } => {
                            *network.phase.lock().unwrap() = ConnectionPhase::Active;
                        }
                        ServerPacket::AuthFailure { reason } => {
                            *network.phase.lock().unwrap() =
                                ConnectionPhase::Failed(format!("Auth failed: code {}", reason));
                        }
                        _ => {}
                    }
                    network.incoming.lock().unwrap().push(packet);
                    consumed
                }
                None => break,
            }
        };
        network.buffer.lock().unwrap().drain(..consumed);
    }
}

// ── Bevy system: drives the handshake state machine ─────────────────────────
pub fn handshake_system(
    network: Res<NetworkConnection>,
    // You'll pass your actual auth token in via a resource
    auth: Res<crate::auth::AuthToken>,
) {
    let mut incoming = network.incoming.lock().unwrap();
    let packets: Vec<ServerPacket> = incoming.drain(..).collect();
    drop(incoming);

    for packet in packets {
        match packet {
            ServerPacket::HandshakeResponse { challenge } => {
                info!("Got challenge, sending auth...");
                if let Err(e) = network.send_auth_request(&auth.token, &challenge) {
                    error!("Failed to send auth request: {}", e);
                }
            }
            ServerPacket::AuthSuccess {
                session_id,
                player_id,
            } => {
                info!(
                    "Authenticated! player_id={} session_id={}",
                    player_id, session_id
                );
            }
            ServerPacket::AuthFailure { reason } => {
                error!("Auth failed with reason code: {:#04x}", reason);
            }
            ServerPacket::Heartbeat => {
                if let Err(e) = network.send_heartbeat_ack() {
                    error!("Failed to send heartbeat ack: {}", e);
                }
            }
            ServerPacket::Unknown { id, .. } => {
                warn!("Unknown packet id: {:#06x}", id);
            }
        }
    }
}
