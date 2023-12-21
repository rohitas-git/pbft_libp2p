
use crate::{
    proposal::{self, EventType, PbftBehaviour, Proposal, Outcome, PbftPeerMetadata},
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
    metadata: &mut PbftPeerMetadata,
) {
    println!(
        "Got message: '{}' with id: {id} from peer: {peer_id}",
        String::from_utf8_lossy(&message.data),
    );

    let event = serde_json::from_slice::<EventType>(&message.data)
        .expect("can get event from gossip message");



    println!("Event recieved: {:?}", event);

    if let EventType::Send(proposal) = event {
        match proposal.stage {
            ConsensusStage::RequestFromClient => handle_request_from_client(swarm, proposal, metadata),
            ConsensusStage::PrePrepare => handle_pre_prepare(swarm, proposal, metadata),
            ConsensusStage::Prepare => handle_prepare(swarm, proposal, metadata),
            ConsensusStage::Commit(vote) => handle_commit(swarm, proposal, metadata, vote),
            ConsensusStage::Result(Outcome::Accepted(valid)) => handle_result(swarm, proposal, metadata, valid),
            _ => (),
        }
    }
}


/// At Request Propagation Stage:
/// When client sends request to Leader,
/// broadcast the Primary proposal to other peers
fn handle_request_from_client(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal, metadata: &mut PbftPeerMetadata) {
    let next_proposal = proposal.to_pre_prepare();
    metadata.set_primary_proposal(&next_proposal);
    let event = EventType::Send(next_proposal);
    let event = serde_json::to_string(&event).expect("can serialize proposal");    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);

    
    if metadata.is_primary(){
        tracing::info!("[REQUEST STAGE]: Primary peer sends Pre-Prepared Proposal to other peers");

        swarm
        .behaviour_mut()
        .gossipsub
        .publish(topic, event.as_bytes()).expect("able to publish client proposal");
    }
    else{
        panic!("Secondary peer can not handle client request");
    }
}


/// At Pre-Prepare Stage:
/// When Leader sends request to other peers,
/// broadcast the Secondary proposal to other peers
fn handle_pre_prepare(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal, metadata: &mut PbftPeerMetadata) {
    let to_send = proposal.to_prepare();
    let event = EventType::Send(to_send);
    let event = serde_json::to_string(&event).expect("can serialize proposal");    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);
    
    tracing::info!("[PRE-PREPARE STAGE]: Secondary peer sends Prepared Proposal to other peers");
    
    // Peers send secondary proposal to other peers
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(topic, event.as_bytes());
}


/// At Prepare Stage:
/// 
fn handle_prepare(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal, metadata: &mut PbftPeerMetadata) {
    let to_send = proposal.to_prepare();
    let event = EventType::Send(to_send);
    let event = serde_json::to_string(&event).expect("can serialize proposal");    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);
    
    // increase count, since recieved a prepare stage message
    metadata.increment_count();

    tracing::info!("[PREPARE STAGE]: Secondary peer sends Prepared Proposal to other peers");
   todo!()

}

fn handle_commit(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal, metadata: &mut PbftPeerMetadata, vote: bool) {
    let to_send = proposal.to_commit(vote);
    let event = EventType::Send(to_send);
    let event = serde_json::to_string(&event).expect("can serialize proposal");
    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);
    
    todo!()
}



fn handle_result(swarm: &mut Swarm<PbftBehaviour>, proposal: Proposal, metadata: &mut PbftPeerMetadata, valid: bool) {
    let to_send = proposal.to_accept_proposal(valid);
    let event = EventType::Send(to_send);
    let event = serde_json::to_string(&event).expect("can serialize proposal");
    let topic = gossipsub::IdentTopic::new(PBFT_TOPIC);
    
    todo!()
}

/// When mdns discovers a peer, we add it to gossipsub's set of explicit peers  
pub fn mdns_discover_peer(swarm: &mut Swarm<PbftBehaviour>, list: Vec<(PeerId, Multiaddr)>) {
    for (peer_id, _multiaddr) in list {
        println!("mDNS discovered a new peer: {peer_id}");
        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    }
}

/// When mdns discovers that a peer has expired, we remove it from gossipsub's set of explicit peers  
pub fn mdns_expired_peer(swarm: &mut Swarm<PbftBehaviour>, list: Vec<(PeerId, Multiaddr)>) {
    for (peer_id, _multiaddr) in list {
        println!("mDNS discover peer has expired: {peer_id}");
        swarm
            .behaviour_mut()
            .gossipsub
            .remove_explicit_peer(&peer_id);
    }
}
