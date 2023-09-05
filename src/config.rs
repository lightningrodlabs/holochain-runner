use holochain::conductor::config::{
    AdminInterfaceConfig, ConductorConfig, InterfaceDriver, KeystoreConfig,
};
use holochain_p2p::kitsune_p2p::{dependencies::url2::Url2, KitsuneP2pConfig, TransportConfig, dependencies::kitsune_p2p_types::config::tuning_params_struct::KitsuneP2pTuningParams};
use holochain_types::db::DbSyncStrategy;
use std::path::PathBuf;
use std::sync::Arc;

pub fn conductor_config(
    admin_port: u16,
    databases_path: &str,
    lair_path: &Option<PathBuf>,
    webrtc_signal_url: &str,
    bootstrap_url: &Url2,
    gossip_arc_clamping: &str,
) -> ConductorConfig {
    // Set network configuration
    let mut network_config = KitsuneP2pConfig::default();
    network_config.bootstrap_service = Some(bootstrap_url.to_owned());
    network_config.transport_pool.push(TransportConfig::WebRTC {
        signal_url: webrtc_signal_url.to_owned(),
    });
    // Set gossip arc clamping
    let mut tuning_params = KitsuneP2pTuningParams::default();
    tuning_params.gossip_arc_clamping = gossip_arc_clamping.into();
    network_config.tuning_params = Arc::new(tuning_params);
    // Build the conductor configuration
    ConductorConfig {
        environment_path: PathBuf::from(databases_path).into(),
        dpki: None,
        db_sync_strategy: DbSyncStrategy::default(),
        keystore: KeystoreConfig::LairServerInProc {
            lair_root: lair_path.to_owned(),
        },
        admin_interfaces: Some(vec![AdminInterfaceConfig {
            driver: InterfaceDriver::Websocket { port: admin_port },
        }]),
        network: Some(network_config),
        tracing_override: None,
    }
}
