use hdk::prelude::AgentPubKey;
use holochain::conductor::{
    api::error::{ConductorApiError, ConductorApiResult},
    ConductorHandle,
};
use holochain_keystore::KeystoreError;
use holochain_p2p::kitsune_p2p::dependencies::kitsune_p2p_types::dependencies::{
    ghost_actor::GhostSender,
    lair_keystore_api_0_0::actor::{
        KeystoreIndex, LairClientApi, LairClientApiSender, LairEntryType,
    },
};

use crate::{emit::emit, StateSignal};
use tokio::sync::mpsc;

pub async fn find_or_generate_key(
    conductor_handle: &ConductorHandle,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
) -> ConductorApiResult<AgentPubKey> {
    let keystore = conductor_handle.keystore().clone();

    let preset_agent_key = match keystore {
        holochain_keystore::MetaLairClient::Legacy(api) => {
            let index_of_entry = api.lair_get_last_entry_index().await.map_err(|_e| {
                ConductorApiError::KeystoreError(KeystoreError::Other(
                    "failed to call lair_get_last_entry_index".to_string(),
                ))
            })?;
            // if last index is 0 there are none
            match index_of_entry.0 {
                0 => None,
                // if last index is greater than 0
                // then lair keystore has some entries
                // loop and check each, picking the first Ed25519 pair
                _ => {
                    let mut option_key_to_return = None;
                    for n in 1..index_of_entry.0 {
                        let keystore_index = KeystoreIndex::from(n);
                        match check_key(keystore_index, api.clone()).await? {
                            Some(key) => {
                                option_key_to_return = Some(key);
                            }
                            // do nothing
                            None => {}
                        }
                    }
                    option_key_to_return
                }
            }
        }
        // we aren't using this version
        holochain_keystore::MetaLairClient::NewLair(_) => unreachable!(),
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

pub async fn check_key(
    index_of_entry: KeystoreIndex,
    api: GhostSender<LairClientApi>,
) -> ConductorApiResult<Option<AgentPubKey>> {
    let entry_type = api
        .lair_get_entry_type(index_of_entry)
        .await
        .map_err(|_e| {
            ConductorApiError::KeystoreError(KeystoreError::Other(
                "failed to call lair_get_entry_type".to_string(),
            ))
        })?;
    match entry_type {
        LairEntryType::SignEd25519 => {
            let public_key = api.sign_ed25519_get(index_of_entry).await.map_err(|_e| {
                ConductorApiError::KeystoreError(KeystoreError::Other(
                    "failed to call sign_ed25519_get".to_string(),
                ))
            })?;
            Ok(Some(AgentPubKey::from_raw_32(public_key.to_vec())))
        }
        _ => return Ok(None),
    }
}
