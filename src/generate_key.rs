use hdk::prelude::AgentPubKey;
use holochain::conductor::{
    api::error::{ConductorApiError, ConductorApiResult},
    ConductorHandle,
};
use holochain_keystore::KeystoreError;
use holochain_p2p::kitsune_p2p::dependencies::kitsune_p2p_types::dependencies::{
    lair_keystore_api::lair_store::*,
};

use crate::{emit::emit, StateSignal};
use tokio::sync::mpsc;

pub async fn find_or_generate_key(
    conductor_handle: &ConductorHandle,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
) -> ConductorApiResult<AgentPubKey> {
    let keystore = conductor_handle.keystore().clone();

    let preset_agent_key = match keystore {
        holochain_keystore::MetaLairClient::Lair(api) => {
            let lair_entries = api.list_entries().await.map_err(|_e| {
                ConductorApiError::KeystoreError(KeystoreError::Other(
                    "failed to call list_entries".to_string(),
                ))
            })?;
            match lair_entries.len() {
                0 => None,
                _ => {
                    // there are lair entries
                    let mut option_key_to_return = None;
                    // we will loop through, and whatever one
                    // is the last one will end up being the one
                    // we choose. Very unselective at the moment, but
                    // that's fine
                    for entry in lair_entries {
                        match entry {
                            LairEntryInfo::Seed { tag: _, seed_info } => {
                                option_key_to_return = Some(AgentPubKey::from_raw_32(
                                    seed_info.ed25519_pub_key.to_vec(),
                                ));
                            }
                            LairEntryInfo::DeepLockedSeed {
                                tag: _,
                                seed_info: _,
                            } => {}
                            LairEntryInfo::WkaTlsCert {
                                tag: _,
                                cert_info: _,
                            } => {}
                            _ => {}
                        }
                    }
                    option_key_to_return
                }
            }
        }
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
                .clone()
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
