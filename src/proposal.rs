use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;


#[derive(Debug,Serialize, Deserialize)]
pub enum ProposalType{
    Primary,
    Secondary,
}

#[derive(Debug,Serialize, Deserialize)]
pub enum EventType{
    SendProposal(ProposalRequest),
    CheckProposal(ProposalRequest),
    Vote(VoteResponse),
}

#[derive(Debug,Serialize, Deserialize)]
pub struct ProposalRequest{
    mode: ProposalType,
    sender: Multiaddr,
    data: Proposal,
}

#[derive(Debug,Serialize, Deserialize)]
pub struct Proposal{
    id: u32,
    client: Multiaddr,
    content: String,
}

#[derive(Debug,Serialize, Deserialize)]
pub struct VoteResponse(bool);