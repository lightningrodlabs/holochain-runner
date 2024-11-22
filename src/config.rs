use holochain::conductor::config::{
    AdminInterfaceConfig, ConductorConfig, DpkiConfig, InterfaceDriver, KeystoreConfig,
};
use holochain::conductor::paths::DataRootPath;
use holochain_keystore::paths::KeystorePath;
use holochain_p2p::kitsune_p2p::dependencies::kitsune_p2p_types::config::{
    KitsuneP2pConfig, TransportConfig,
};
use holochain_p2p::kitsune_p2p::{
    dependencies::kitsune_p2p_types::config::tuning_params_struct::KitsuneP2pTuningParams,
    dependencies::url2::Url2,
};
use holochain_types::db::DbSyncStrategy;
use holochain_types::websocket::AllowedOrigins;
use std::path::PathBuf;
use std::sync::Arc;

pub fn conductor_config(
    admin_port: u16,
    databases_path: PathBuf,
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
        webrtc_config: None,
    });
    // Set gossip arc clamping
    let mut tuning_params = KitsuneP2pTuningParams::default();
    tuning_params.gossip_arc_clamping = gossip_arc_clamping.into();
    network_config.tuning_params = Arc::new(tuning_params);
    // Build the conductor configuration
    ConductorConfig {
        dpki: DpkiConfig {
            dna_path: None,
            network_seed: "".to_string(),
            allow_throwaway_random_dpki_agent_key: false,
            no_dpki: true,
        },
        db_sync_strategy: DbSyncStrategy::default(),
        keystore: KeystoreConfig::LairServerInProc {
            lair_root: lair_path
                .as_ref()
                .map(|path_buf| KeystorePath::from(path_buf.clone())),
        },
        admin_interfaces: Some(vec![AdminInterfaceConfig {
            driver: InterfaceDriver::Websocket {
                port: admin_port,
                allowed_origins: AllowedOrigins::Any,
            },
        }]),
        network: network_config,
        tracing_override: None,
        data_root_path: Some(DataRootPath::from(databases_path)),
        tuning_params: None,
        chc_url: None,
        device_seed_lair_tag: None,
        danger_generate_throwaway_device_seed: false,
    }
}
