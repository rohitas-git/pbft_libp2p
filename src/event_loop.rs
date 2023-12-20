use crate::handler::*;
use crate::proposal::{PbftBehaviour, PbftBehaviourEvent, EventType};
use futures::stream::StreamExt;
pub use libp2p::{gossipsub, mdns, swarm::NetworkBehaviour, swarm::SwarmEvent, Swarm};
pub use tokio::{io, io::AsyncBufReadExt, select};

pub async fn run(mut swarm: Swarm<PbftBehaviour>) {
    // assumed peer is not Leader (primary)
    let mut is_primary = false;

    // check if cli says that peer is primary
    if let Some(peer_role) = std::env::args().nth(1) {
        is_primary = peer_role == *"primary".to_string();
    }

    // create topic and subscribe to it
    let topic = gossipsub::IdentTopic::new("pbft-net");
    swarm.behaviour_mut().gossipsub.subscribe(&topic);

    // If primary peer, fetch client (request) proposal
    // And broadcast it as primary proposal to other peers 
    if is_primary {
        let given_proposal = crate::proposal::get_client_proposal().to_pre_prepare();
        let event = EventType::Send(given_proposal);

        if let Ok(data) = serde_json::to_string(&event) {
            swarm
                .behaviour_mut()
                .gossipsub
                .publish(topic, data.as_bytes());
        }
    }

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
                })) => handle_gossip_message(&mut swarm, peer_id, id, message),
                _ => ()
            }
        }
    }
}

pub async fn sample_run(mut swarm: Swarm<PbftBehaviour>) {
    loop {
        select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(PbftBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    mdns_discover_peer(&mut swarm, list);
                },
                SwarmEvent::Behaviour(PbftBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    mdns_expired_peer(&mut swarm, list);
                },
                SwarmEvent::Behaviour(PbftBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => handle_gossip_message(&mut swarm, peer_id, id, message),
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                }
                _ => {}
            }
        }
    }
}
