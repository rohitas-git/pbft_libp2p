use crate::handler::*;
use crate::proposal::{EventType, PbftBehaviour, PbftBehaviourEvent, PbftPeerMetadata, Proposal};
use futures::stream::StreamExt;
pub use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux, Swarm,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
pub use tokio::{io, io::AsyncBufReadExt, select};

pub fn setup(swarm:&mut Swarm<PbftBehaviour>) -> Result<PbftPeerMetadata, Box<dyn Error>> {
    let mut metadata = PbftPeerMetadata::new();

    // check if cli says that peer is primary
    if let Some(peer_role) = std::env::args().nth(1) {
        if peer_role == *"primary".to_string() {
            metadata.set_primary();
            println!("Primary Peer's Id: {}", swarm.local_peer_id());

        }
    }
    println!("Secondary Peer's Id: {}", swarm.local_peer_id());


    // create topic and subscribe to it
    let topic = gossipsub::IdentTopic::new("pbft-net");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // If primary peer, fetch client (request) proposal
    // And broadcast it as primary proposal to other peers
    if metadata.is_primary() {
        let given_proposal = crate::proposal::get_client_proposal().to_pre_prepare();
        metadata.set_primary_proposal(&given_proposal);

        let event = EventType::Send(given_proposal);
        if let Ok(data) = serde_json::to_string(&event) {
            swarm
                .behaviour_mut()
                .gossipsub
                .publish(topic, data.as_bytes())?;
        }
    }

    Ok(metadata)
}

pub async fn run(
    mut swarm: Swarm<PbftBehaviour>,
    metadata: &mut PbftPeerMetadata,
) -> Result<(), Box<dyn Error>> {

    // event loop for swarm state machine kicks off
    loop {
        select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(PbftBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    mdns_discover_peer(&mut swarm, list);
                },
                SwarmEvent::Behaviour(PbftBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    mdns_expired_peer(&mut swarm, list)
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                },
                SwarmEvent::Behaviour(PbftBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => handle_gossip_message(&mut swarm, peer_id, id, message, metadata),
                _ => ()
            }
        }
    }
}

pub fn create_swarm() -> Result<Swarm<PbftBehaviour>, Box<dyn Error>> {
    let swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            // To content-address message, we can take the hash of message and use it as an ID.
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            // Set a custom gossipsub configuration
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

            // build a gossipsub network behaviour
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(PbftBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Ok(swarm)
}
