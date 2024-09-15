use crate::chain::block::Block;
use crate::chain::Chain;
use crate::protocol::command::{ChainRequest, ChainResponse};
use crate::protocol::error::ProtocolError;
use crate::protocol::network::{Network, NetworkEvent};
use libp2p::futures::stream::SelectNextSome;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::IdentTopic;
use libp2p::swarm::SwarmEvent;
use libp2p::{gossipsub, mdns, Swarm};

mod command;
mod error;
mod network;

pub struct Protocol {
    network: Swarm<Network>,
    topic: IdentTopic,
    chain_count: usize,
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

        Ok(Protocol {
            network,
            topic,
            chain_count: 0,
        })
    }

    pub fn fetch_network_event(&mut self) -> SelectNextSome<'_, Swarm<Network>> {
        self.network.select_next_some()
    }

    pub fn handle_network_event(&mut self, chain: &mut Chain, event: SwarmEvent<NetworkEvent>) -> Result<String, ProtocolError> {
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
            SwarmEvent::Behaviour(NetworkEvent::Channel(gossipsub::Event::Message { message, .. })) => {
                // Received whole chain from peer - usually on startup
                if let Ok(remote_chain) = serde_json::from_slice::<ChainResponse>(&message.data) {
                    if *self.network.local_peer_id() == remote_chain.to_peer {
                        chain
                            .replace_chain(&remote_chain.chain)
                            .map_err(|error| ProtocolError::ChainError(error))?;

                        return Ok(format!(
                            "Chain replaced with new chain {}",
                            serde_json::to_string(&remote_chain.chain).unwrap()
                        ));
                    }
                }
                // Got request from peer for chain - usually on peer startup
                else if let Ok(chain_request) = serde_json::from_slice::<ChainRequest>(&message.data) {
                    if *self.network.local_peer_id() == chain_request.from_peer {
                        self.publish_chain(chain)?;

                        return Ok(format!("Chain published to peer {}", chain_request.from_peer));
                    }
                }
                // Received new block
                else if let Ok(block) = serde_json::from_slice::<Block>(&message.data) {
                    if chain.add_new_block(block.clone()).is_ok() {
                        // Relaying block
                        self.publish_block(&block)?;

                        return Ok(format!("Block added to chain {}", serde_json::to_string(&block).unwrap()));
                    }
                }
            }
            _ => {}
        }

        Ok("NOP".to_string())
    }

    pub fn chain_contains_changes(&self, chain: &Chain) -> bool {
        chain.blocks.len() > self.chain_count
    }

    pub fn publish_changes(&mut self, chain: &Chain) -> Result<(), ProtocolError> {
        if self.chain_contains_changes(chain) {
            self.publish_block(&chain.blocks.last().unwrap())?;

            self.chain_count = chain.blocks.len();
        }

        Ok(())
    }

    fn publish_block(&mut self, block: &Block) -> Result<(), ProtocolError> {
        let topic = &self.topic;

        let block = serde_json::to_string(block).map_err(|error| ProtocolError::ParseError(error.to_string()))?;

        self.network
            .behaviour_mut()
            .channel
            .publish(topic.clone(), block.as_bytes())
            .map_err(|error| ProtocolError::PublishingError(error.to_string()))?;

        Ok(())
    }

    fn publish_chain(&mut self, chain: &Chain) -> Result<(), ProtocolError> {
        let topic = &self.topic;
        let blockchain = serde_json::to_string(&chain.blocks).map_err(|error| ProtocolError::ParseError(error.to_string()))?;

        self.network
            .behaviour_mut()
            .channel
            .publish(topic.clone(), blockchain.as_bytes())
            .map_err(|error| ProtocolError::PublishingError(error.to_string()))?;

        Ok(())
    }
}
