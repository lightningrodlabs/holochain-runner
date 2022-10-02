use hdk::prelude::AgentPubKey;
use holochain::conductor::{api::error::ConductorApiResult, ConductorHandle};

use crate::{emit::emit, StateSignal};
use tokio::sync::mpsc;

pub async fn find_or_generate_key(
    conductor_handle: &ConductorHandle,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
) -> ConductorApiResult<AgentPubKey> {
    let public_keys = conductor_handle.keystore().list_public_keys().await?;
    let preset_agent_key = if public_keys.len() > 0 {
        Some(public_keys.first().unwrap().to_owned())
    } else {
        None
    };

    match preset_agent_key {
        Some(agent_key) => {
            println!("Recognized a keypair, using that...");
            Ok(agent_key)
        }
        None => {
            emit(event_channel, StateSignal::CreatingKeys).await;
            println!("Don't recognize you, so generating a new identity for you...");
            let agent_key = conductor_handle
                .keystore()
                .new_sign_keypair_random()
                .await?;
            emit(event_channel, StateSignal::RegisteringDna).await;
            println!(
                "Your new key pair is generated, the public key is: {:?}",
                agent_key
            );
            Ok(agent_key)
        }
    }
}
