use std::{collections::HashMap, path::PathBuf};

use hdk::prelude::{AgentPubKey, Uid};
use holochain::conductor::{
    api::error::{ConductorApiError, ConductorApiResult, SerializationError},
    error::ConductorError,
    CellError, ConductorHandle,
};
use holochain_types::prelude::{InstallAppBundlePayload, AppBundleSource};
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
    happ_path: PathBuf,
    // membrane_proof: Option<String>,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
    uid: Option<Uid>,
) -> ConductorApiResult<()> {
    
    println!("continuing with the installation...");
    // register any dnas
    emit(event_channel, StateSignal::InstallingApp).await;
    // Install the CellIds as an "app", with an installed_app_id
    let payload: InstallAppBundlePayload = InstallAppBundlePayload {
        source: AppBundleSource::Path(happ_path),
        agent_key,
        installed_app_id: Some(app_id),
        membrane_proofs: HashMap::new(),
        uid,
    };
    conductor_handle
        .clone()
        // .install_app(app_id, cell_ids_with_proofs.clone())
        .install_app_bundle(payload)
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
