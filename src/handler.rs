use crate::{
    proposal::{self, EventType, PbftBehaviour, Proposal},
    PBFT_TOPIC,
};
use libp2p::{
    gossipsub::{self, Message, MessageId},
    Multiaddr, PeerId, Swarm,
};

use proposal::ConsensusStage;

pub fn handle_gossip_message(
    swarm: &mut Swarm<PbftBehaviour>,
    peer_id: PeerId,
    id: MessageId,
    message: Message,
) {
    println!(
        "Got message: '{}' with id: {id} from peer: {peer_id}",
        String::from_utf8_lossy(&message.data),
    );

    let event = serde_json::from_slice::<EventType>(&message.data)
        .expect("can get event from gossip message");

    if let EventType::Send(proposal) = event {
        match proposal.mode {
            ConsensusStage::PrePrepare => handle_pre_prepare(swarm, proposal),
            ConsensusStage::Prepare => (),
            ConsensusStage::Commit(vote) => ha
            _ => (),
        }
    }
}


/// At Request Propagation Stage:
/// When client sends request to Leader,
/// broadcast the Primary proposal to other peers
fn handle_request_from_client(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal) {
    let to_send = proposal.to_pre_prepare();
    let to_send = serde_json::to_string(&to_send).expect("can serialize proposal");
    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);

    swarm
        .behaviour_mut()
        .gossipsub
        .publish(topic, to_send.as_bytes()).expect("able to publish client proposal");
}

/// At Pre-Prepare Stage:
/// When Leader sends request to other peers,
/// broadcast the Secondary proposal to other peers
fn handle_pre_prepare(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal) {
    let to_send = proposal.to_prepare();
    let to_send = serde_json::to_string(&to_send).expect("can serialize proposal");
    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);
    
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(topic, to_send.as_bytes());
}

fn handle_commit(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal, vote: bool) {
    let to_send = proposal.to_commit(vote);
    let event = EventType::Send(to_send);
    let event = serde_json::to_string(&event).expect("can serialize proposal");
    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);
    
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(topic, event.as_bytes());
}

fn handle_result(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal, vote: bool) {
    let to_send = proposal.to_accept_proposal(vote);
    let event = EventType::Send(to_send);
    let event = serde_json::to_string(&event).expect("can serialize proposal");
    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);
    
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(topic, event.as_bytes());
}

pub fn mdns_discover_peer(swarm: &mut Swarm<PbftBehaviour>, list: Vec<(PeerId, Multiaddr)>) {
    for (peer_id, _multiaddr) in list {
        println!("mDNS discovered a new peer: {peer_id}");
        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    }
}

pub fn mdns_expired_peer(swarm: &mut Swarm<PbftBehaviour>, list: Vec<(PeerId, Multiaddr)>) {
    for (peer_id, _multiaddr) in list {
        println!("mDNS discover peer has expired: {peer_id}");
        swarm
            .behaviour_mut()
            .gossipsub
            .remove_explicit_peer(&peer_id);
    }
}
