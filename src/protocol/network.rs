use libp2p::{gossipsub, mdns, noise, swarm::NetworkBehaviour, tcp, yamux, Swarm, SwarmBuilder};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::error::Error;
use std::option::Option;
use std::time::Duration;
use tokio::io;

#[derive(NetworkBehaviour)]
pub struct Network {
    pub channel: gossipsub::Behaviour,
    pub address_resolver: mdns::tokio::Behaviour,
}

impl Network {
    pub fn init() -> Result<Swarm<Network>, Box<dyn Error>> {
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_behaviour(|key| {
                let gossip_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(|message| {
                        let mut hasher = DefaultHasher::new();
                        message.data.hash(&mut hasher);
                        gossipsub::MessageId::from(hasher.finish().to_string())
                    })
                    .build()
                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

                Ok(Network {
                    channel: gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(key.clone()), gossip_config)?,
                    address_resolver: mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?,
                })
            })?
            .with_swarm_config(|config| config.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(swarm)
    }
}
