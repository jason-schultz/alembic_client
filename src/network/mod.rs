pub mod connection;
pub mod events;
pub mod packets;
pub mod systems;

pub use connection::NetworkConnection;
pub use events::{NetworkCommand, NetworkEvent, TileData};

use bevy::prelude::*;
use systems::{handle_network_commands, poll_network, process_packets};

use crate::network::packets::ServerPacket;

#[derive(Resource, Default)]
pub struct PacketQueue(pub Vec<ServerPacket>);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkConnection::default())
            .insert_resource(PacketQueue::default())
            .add_event::<NetworkCommand>()
            .add_event::<NetworkEvent>()
            .add_systems(
                Update,
                (
                    handle_network_commands,
                    poll_network,
                    // process_packets must run after poll_network so packets
                    // parsed this frame are handled this frame.
                    process_packets.after(poll_network),
                ),
            );
    }
}
