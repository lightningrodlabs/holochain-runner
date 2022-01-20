use hdk::prelude::{AgentPubKey, Uid};
use holochain::conductor::{
    api::error::{ConductorApiError, ConductorApiResult, SerializationError},
    error::ConductorError,
    CellError, ConductorHandle,
};
#[allow(deprecated)]
use holochain_types::{
    app::InstalledAppId,
    prelude::{DnaBundle, InstalledCell},
};
use holochain_zome_types::{CellId, SerializedBytes, SerializedBytesError, UnsafeBytes};
use tokio::sync::mpsc;

use crate::emit::{emit, StateSignal};

pub async fn install_app(
    conductor_handle: &ConductorHandle,
    agent_key: AgentPubKey,
    app_id: InstalledAppId,
    dnas: Vec<(Vec<u8>, String)>,
    membrane_proof: Option<String>,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
    uid: Option<Uid>,
) -> ConductorApiResult<()> {
    
    println!("continuing with the installation...");
    // register any dnas
    let tasks = dnas.into_iter().map(|(dna_bytes, nick)| {
        let agent_key = agent_key.clone();
        let conductor_handle_clone = conductor_handle.clone();
        let proof_cloned = membrane_proof.clone();
        let uid_cloned = uid.clone();
        tokio::task::spawn(async move {
            println!("decoding dna bundle");
            let dna = DnaBundle::decode(&dna_bytes)?;
            println!("converting to dna file");
            let (dna_file, _original_dna_hash) = dna.into_dna_file(uid_cloned, None).await?;
            println!("calling register dna");
            conductor_handle_clone.register_dna(dna_file.clone()).await?;
            let cell_id = CellId::from((dna_file.dna_hash().clone(), agent_key));

            // if there's a membrane proof
            // decode it from base64 using default options
            // and construct SerializedBytes from it
            let membrane_proof = match proof_cloned {
                Some(string_proof) => match base64::decode(string_proof) {
                    Ok(res) => {
                        let unsafe_bytes = UnsafeBytes::from(res);
                        Some(SerializedBytes::from(unsafe_bytes))
                    }
                    Err(_e) => {
                        return Err(ConductorApiError::SerializationError(
                            SerializationError::Bytes(SerializedBytesError::Deserialize(
                                "couldnt decode base64".to_string(),
                            )),
                        ))
                    }
                },
                None => None,
            };
            #[allow(deprecated)]
            ConductorApiResult::Ok((InstalledCell::new(cell_id, nick), membrane_proof))
        })
    });
    // Join all the install tasks
    let cell_ids_with_proofs = futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|result| result.unwrap())
        // Check all passed and return the proofs
        .collect::<Result<Vec<_>, _>>()?;
    emit(event_channel, StateSignal::InstallingApp).await;
    // Install the CellIds as an "app", with an installed_app_id
    conductor_handle
        .clone()
        .install_app(app_id, cell_ids_with_proofs.clone())
        .await?;
    Ok(())
}

pub async fn enable_app(
    conductor_handle: &ConductorHandle,
    app_id: InstalledAppId,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
) -> ConductorApiResult<()> {
    // relates to: https://github.com/holochain/holochain/blob/55af424c2f2c2669d8253804f4e2b888abf245f2/crates/holochain/src/conductor/api/api_external/admin_interface.rs
    // Enable app
    emit(event_channel, StateSignal::EnablingApp).await;
    let (_app, mut errors) = conductor_handle.clone().enable_app(app_id.clone()).await?;
    conductor_handle
        .get_app_info(&app_id)
        .await?
        .ok_or(ConductorError::AppNotInstalled(app_id))?;
    if errors.len() > 0 {
        if let Some((_cell_id, cell_error)) = errors.pop() {
            Err(cell_error.into())
        } else {
            Err(ConductorApiError::CellError(CellError::Todo))
        }
    } else {
        Ok(())
    }
}
