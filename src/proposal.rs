use libp2p::{gossipsub, Multiaddr, mdns};
use serde::{Deserialize, Serialize};
use libp2p::swarm::NetworkBehaviour;
use tokio::{fs, io::AsyncBufReadExt, sync::mpsc};


type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[derive(Debug,Serialize, Deserialize)]
pub struct Proposal{

    /// proposal id
    id: u32,

    /// during dev, client is the primary peer itself
    client: Multiaddr,

    /// the content of proposal
    content: String,
}


/// ProposalMode is an enum that is used for matching the types of messages passed.
/// ProposalMode is used to determine how the message is processed.
#[derive(Debug,Serialize, Deserialize)]
pub enum ProposalMode{

    /// when message is sent by primary to other peers
    PrimaryBroadcast,

    /// when message is sent by secondary to other peers
    SecondaryBroadcast,

    /// when each peer broadcast their votes to others
    Vote(bool),
    
    /// Sent by primary to client when consensus is reached
    Valid(bool),

    /// Sent by primary to client when consensus is not reachable
    Invalid,
}

/// Struct to denote a proposal request. It contains a reference to the [PeerMode] enum.
/// and an optional Proposal object.
#[derive(Debug,Serialize, Deserialize)]
pub struct ProposalRequest{
    mode: ProposalMode,
    sender: Multiaddr,
    data: Proposal,
}

#[derive(Debug,Serialize, Deserialize)]
pub struct ProposalResponse{
    mode: ProposalMode,
    sender: Multiaddr,
    data: Proposal,
}


/// an enum to handle responses from peers 
pub enum EventType {

    /// for denoting a response from peers
    Response(ProposalResponse),

    /// denoting an input from stdin.
    Input(String),
}


/// This struct is used to handle the network behaviour between peers.
#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
struct MemoBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    #[behaviour(ignore)]
    response_sender: mpsc::UnboundedSender<ProposalResponse>,
}
