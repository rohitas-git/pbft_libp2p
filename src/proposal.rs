use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, mdns, Multiaddr};
use serde::{Deserialize, Serialize};

type Valid = bool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    /// stage of proposal
    pub mode: ConsensusStage,
    /// during dev, client is the primary peer itself
    client: String,
    /// the content of proposal
    content: String,
}

impl Proposal {
    pub fn new_request(client: String, content: String) -> Self {
        Proposal {
            mode: ConsensusStage::RequestFromClient,
            client,
            content,
        }
    }

    pub fn to_pre_prepare(self) -> Self {
        Proposal {
            mode: ConsensusStage::PrePrepare,
            ..self
        }
    }

    pub fn to_prepare(self) -> Self {
        Proposal {
            mode: ConsensusStage::Prepare,
            ..self
        }
    }

    pub fn to_commit(self, vote: Valid) -> Self {
        Proposal {
            mode: ConsensusStage::Commit(vote),
            ..self
        }
    }

    pub fn to_accept_proposal(self, vote: Valid) -> Self {
        Proposal {
            mode: ConsensusStage::Result(Outcome::Accepted(vote)),
            ..self
        }
    }

    pub fn to_reject_proposal(self, vote: Valid) -> Self {
        Proposal {
            mode: ConsensusStage::Result(Outcome::Rejected),
            ..self
        }
    }
}

/// ProposalMode is an enum that is used for matching the types of messages passed.
/// ProposalMode is used to determine how the message is processed.
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
