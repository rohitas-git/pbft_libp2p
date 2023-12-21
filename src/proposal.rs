use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, mdns, Multiaddr, Swarm};
use serde::{Deserialize, Serialize};

type Valid = bool;

/// Members: stage, ClientAddr, Content
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proposal {
    /// stage of proposal
    pub stage: ConsensusStage,
    /// during dev, client is the primary peer itself
    client: String,
    /// the content of proposal
    content: String,
}

impl Proposal {
    /// Create a new request from client with some content
    pub fn new_request(client: String, content: String) -> Self {
        Proposal {
            stage: ConsensusStage::RequestFromClient,
            client,
            content,
        }
    }

    /// Change the stage to PrePrepare
    pub fn to_pre_prepare(self) -> Self {
        Proposal {
            stage: ConsensusStage::PrePrepare,
            ..self
        }
    }

    /// Change the stage to Prepare
    pub fn to_prepare(self) -> Self {
        Proposal {
            stage: ConsensusStage::Prepare,
            ..self
        }
    }

    /// Change the stage to Commit(vote)
    pub fn to_commit(self, vote: Valid) -> Self {
        Proposal {
            stage: ConsensusStage::Commit(vote),
            ..self
        }
    }

    /// Change the stage to Result(Outcome::Accepted(valid))
    pub fn to_accept_proposal(self, valid: Valid) -> Self {
        Proposal {
            stage: ConsensusStage::Result(Outcome::Accepted(valid)),
            ..self
        }
    }

    /// Change the stage to Result(Outcome::Rejected)
    pub fn to_reject_proposal(self) -> Self {
        Proposal {
            stage: ConsensusStage::Result(Outcome::Rejected),
            ..self
        }
    }
}

/// ProposalMode is an enum that is used for matching the types of messages passed.
/// ProposalMode is used to determine how the message is processed.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConsensusStage {
    /// a client sending a request to the primary (leader) node in the pBFT network.
    RequestFromClient,
    /// The primary node (leader) creates a pre-prepare message containing the proposed request
    /// and sends it to all other nodes.
    PrePrepare,
    /// Upon receiving the pre-prepare message, other nodes validate the request
    /// and broadcast a prepare message to all nodes, indicating that they accept the request.
    Prepare,
    ///  Once a node collects prepare messages from a two-thirds majority of nodes (including itself),
    /// it sends a commit message to all nodes, finalizing the agreement.
    Commit(Valid),
    /// The primary node (leader) collects commit messages from a two-thirds majority of nodes
    /// and sends a response to the client.
    Result(Outcome),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Outcome {
    Accepted(Valid),
    Rejected,
}

/// an enum to handle responses from peers
#[derive(Debug, Serialize, Deserialize)]
pub enum EventType {
    /// for denoting a response from peers
    Send(Proposal),
    // for denoting a request from peers
    // Outgoing(Proposal),
}

/// This struct is used to handle the network behaviour between peers.
#[derive(NetworkBehaviour)]
pub struct PbftBehaviour {
    /// to handle peer gossip
    pub gossipsub: gossipsub::Behaviour,
    /// to handle peer discovery
    pub mdns: mdns::tokio::Behaviour,
}

pub fn get_client_proposal() -> Proposal {
    let input_path = "client.json".to_string();
    let input = std::fs::read_to_string(input_path).expect("able to get client proposal");
    let given_proposal =
        serde_json::from_str::<Proposal>(&input).expect("can jsonify client proposal");

    given_proposal
}

#[derive(Debug)]
pub struct PbftPeerMetadata {
    // pbft_peers_count: u32,
    in_favor: u32,
    opp_favor: u32,
    is_primary: bool,
    primary_proposal: Option<Proposal>,
    secondary_proposals: Vec<Proposal>,
    recipient_count: u32,
}

impl PbftPeerMetadata {
    pub fn new() -> Self {
        PbftPeerMetadata {
            in_favor: 0,
            opp_favor: 0,
            is_primary: false,
            primary_proposal: None,
            secondary_proposals: Vec::new(),
            recipient_count: 0,
        }
    }

    pub fn increment_count(&mut self){
        self.recipient_count +=1;
    }

    pub fn set_primary_proposal(&mut self, proposal: &Proposal){
        if self.primary_proposal.is_some(){
            println!("Overwriting the primary proposal: {:?} ", self.primary_proposal.clone().unwrap());
        }
        else{
            self.primary_proposal = Some(proposal.clone());
        }
    }

    pub fn add_secondary_proposal(&mut self, sec_proposal: &Proposal){
        self.secondary_proposals.push(sec_proposal.clone());
    }

    pub fn set_primary(&mut self) {
        self.is_primary = true;
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    pub fn reset(&mut self) {
        self.in_favor = 0;
        self.opp_favor = 0;
        self.primary_proposal = None;
        self.secondary_proposals = Vec::new();
    }


}
