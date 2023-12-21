mod event_loop;
mod handler;
mod proposal;
mod protocol;

use std::error::Error;
use tracing_subscriber::EnvFilter;

pub const PBFT_TOPIC: &str = "pbft-net";

// 6 seconds
const PBFT_TIME: u32 = 6;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let mut swarm = event_loop::create_swarm()?;
    
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    
    let metadata = &mut event_loop::setup(&mut swarm)?;
    
    event_loop::run(swarm, metadata).await?;

    Ok(())
}
