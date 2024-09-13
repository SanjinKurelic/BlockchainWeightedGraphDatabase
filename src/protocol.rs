use crate::protocol::error::ProtocolError;
use crate::protocol::network::{Network, NetworkEvent};
use libp2p::futures::stream::SelectNextSome;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::IdentTopic;
use libp2p::swarm::SwarmEvent;
use libp2p::{gossipsub, mdns, Swarm};
use tokio::io::AsyncBufReadExt;

mod error;
mod network;

pub struct Protocol {
    network: Swarm<Network>,
    topic: IdentTopic,
}

impl Protocol {
    const COMMAND_TOPIC: &'static str = "command";

    pub fn init() -> Result<Protocol, ProtocolError> {
        let mut network = Network::init().map_err(|error| ProtocolError::NetworkError(error.to_string()))?;
        let topic = IdentTopic::new(Self::COMMAND_TOPIC);

        network
            .behaviour_mut()
            .channel
            .subscribe(&topic)
            .map_err(|error| ProtocolError::NetworkError(error.to_string()))?;

        Ok(Protocol { network, topic })
    }

    pub fn fetch_network_event(&mut self) -> SelectNextSome<'_, Swarm<Network>> {
        self.network.select_next_some()
    }

    pub fn handle_network_event(&mut self, event: SwarmEvent<NetworkEvent>) {
        match event {
            SwarmEvent::Behaviour(NetworkEvent::AddressResolver(mdns::Event::Discovered(list))) => {
                for (peer_id, _multiaddr) in list {
                    self.network.behaviour_mut().channel.add_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(NetworkEvent::AddressResolver(mdns::Event::Expired(list))) => {
                for (peer_id, _multiaddr) in list {
                    self.network.behaviour_mut().channel.remove_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(NetworkEvent::Channel(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                // TODO update local chain
                print!("Received {}", String::from_utf8_lossy(&message.data));
            }
            _ => {}
        }
    }
}
