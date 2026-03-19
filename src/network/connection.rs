use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::network::packets::{
    AUTH_REQUEST, HANDSHAKE_REQUEST, HEARTBEAT_ACK, JOIN_WORLD, LEAVE_WORLD, MAGIC, PLAYER_MOVE,
    VERSION,
};

/// The phase of the network connection state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionPhase {
    /// No connection attempt has been made yet.
    Disconnected,
    /// TCP connected, handshake sent, waiting for challenge.
    AwaitingChallenge,
    /// Challenge received, auth request sent, waiting for result.
    Authenticating,
    /// Fully authenticated and active.
    Active,
    /// Auth failed or an IO error occurred.
    Failed(String),
}

/// Shared network state held as a Bevy resource.
/// The inner fields use `Arc<Mutex<_>>` so the resource can be cloned
/// and sent to a background thread for the initial TCP connect.
#[derive(bevy::prelude::Resource, Clone)]
pub struct NetworkConnection {
    pub stream: Arc<Mutex<Option<TcpStream>>>,
    pub phase: Arc<Mutex<ConnectionPhase>>,
    /// Accumulation buffer for partial incoming packets.
    pub buffer: Arc<Mutex<Vec<u8>>>,
}

impl Default for NetworkConnection {
    fn default() -> Self {
        Self {
            stream: Arc::new(Mutex::new(None)),
            phase: Arc::new(Mutex::new(ConnectionPhase::Disconnected)),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl NetworkConnection {
    /// Establish a TCP connection to `addr` and set the stream to non-blocking.
    /// Called from a background thread so it does not block the Bevy main loop.
    pub fn connect(&self, addr: &str) -> std::io::Result<()> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        *self.stream.lock().unwrap() = Some(stream);
        Ok(())
    }

    pub fn phase(&self) -> ConnectionPhase {
        self.phase.lock().unwrap().clone()
    }

    pub fn is_active(&self) -> bool {
        *self.phase.lock().unwrap() == ConnectionPhase::Active
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(
            *self.phase.lock().unwrap(),
            ConnectionPhase::Disconnected | ConnectionPhase::Failed(_)
        )
    }

    pub fn set_phase(&self, phase: ConnectionPhase) {
        *self.phase.lock().unwrap() = phase;
    }

    // ── Outgoing packets ──────────────────────────────────────────────────

    /// Send handshake request and transition to AwaitingChallenge.
    pub fn send_handshake_request(&self, client_id: &str) -> std::io::Result<()> {
        let id_bytes = client_id.as_bytes();
        let mut payload = Vec::new();
        payload.extend_from_slice(&(id_bytes.len() as u16).to_be_bytes());
        payload.extend_from_slice(id_bytes);
        payload.extend_from_slice(&0x0100u16.to_be_bytes()); // protocol version

        self.send_raw(HANDSHAKE_REQUEST, &payload)?;
        self.set_phase(ConnectionPhase::AwaitingChallenge);
        Ok(())
    }

    /// Send auth request with HMAC and transition to Authenticating.
    pub fn send_auth_request(&self, token: &str, challenge: &[u8; 32]) -> std::io::Result<()> {
        let token_bytes = token.as_bytes();
        let hmac = compute_hmac(token_bytes, challenge);

        let mut payload = Vec::new();
        payload.extend_from_slice(&(token_bytes.len() as u16).to_be_bytes());
        payload.extend_from_slice(token_bytes);
        payload.extend_from_slice(&hmac);

        self.send_raw(AUTH_REQUEST, &payload)?;
        self.set_phase(ConnectionPhase::Authenticating);
        Ok(())
    }

    /// Send heartbeat acknowledgement.
    pub fn send_heartbeat_ack(&self) -> std::io::Result<()> {
        self.send_raw(HEARTBEAT_ACK, &[])
    }

    pub fn send_join_world(&self, world_id: &str) -> std::io::Result<()> {
        let id_bytes = world_id.as_bytes();
        let mut payload = Vec::new();
        payload.extend_from_slice(&(id_bytes.len() as u16).to_be_bytes());
        payload.extend_from_slice(id_bytes);
        self.send_raw(JOIN_WORLD, &payload)
    }

    pub fn send_leave_world(&self, world_id: &str) -> std::io::Result<()> {
        let id_bytes = world_id.as_bytes();
        let mut payload = Vec::new();
        payload.extend_from_slice(&(id_bytes.len() as u16).to_be_bytes());
        payload.extend_from_slice(id_bytes);
        self.send_raw(LEAVE_WORLD, &payload)
    }

    pub fn send_player_move(&self, x: u16, y: u16, facing: u8) -> std::io::Result<()> {
        let payload = [(x >> 8) as u8, x as u8, (y >> 8) as u8, y as u8, facing];
        self.send_raw(PLAYER_MOVE, &payload)
    }

    fn send_raw(&self, packet_id: u16, payload: &[u8]) -> std::io::Result<()> {
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

// ── HMAC-SHA256(key=token, msg=challenge) ─────────────────────────────────
fn compute_hmac(token: &[u8], challenge: &[u8; 32]) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(token).expect("HMAC accepts any key length");
    mac.update(challenge);
    mac.finalize().into_bytes().into()
}
