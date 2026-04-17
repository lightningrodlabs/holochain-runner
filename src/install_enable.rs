use crate::emit::{emit, StateSignal};
use holochain_client::{AdminWebsocket, EnableAppResponse};
use holochain_types::app::InstalledAppId;
use holochain_types::prelude::{AgentPubKey, NetworkSeed};
use holochain_types::prelude::{AppBundleSource, InstallAppPayload};
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::mpsc;

pub async fn install_app(
    admin_client: &AdminWebsocket,
    agent_key: AgentPubKey,
    app_id: InstalledAppId,
    happ_path: PathBuf,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
    network_seed: Option<NetworkSeed>,
) -> anyhow::Result<()> {
    emit(event_channel, StateSignal::InstallingApp).await;
    admin_client
        .install_app(InstallAppPayload {
            source: AppBundleSource::Path(happ_path),
            agent_key: Some(agent_key),
            installed_app_id: Some(app_id),
            roles_settings: Some(HashMap::new()),
            network_seed,
            ignore_genesis_failure: false,
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to install app {:?}", e))?;
    Ok(())
}

pub async fn enable_app(
    admin_client: &AdminWebsocket,
    app_id: InstalledAppId,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
) -> anyhow::Result<()> {
    emit(event_channel, StateSignal::EnablingApp).await;
    let _enable_response: EnableAppResponse = admin_client
        .enable_app(app_id.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to enable app {:?}", e))?;

    Ok(())
}
