use holochain::conductor::config::{
    AdminInterfaceConfig, ConductorConfig, InterfaceDriver, KeystoreConfig, NetworkConfig,
};
use holochain::conductor::paths::DataRootPath;
use holochain_keystore::paths::KeystorePath;
use holochain_types::db::DbSyncStrategy;
use holochain_types::websocket::AllowedOrigins;
use std::path::PathBuf;
use url2::Url2;

pub fn conductor_config(
    admin_port: u16,
    databases_path: PathBuf,
    lair_path: &Option<PathBuf>,
    webrtc_signal_url: &str,
    bootstrap_url: &Url2,
    relay_url: &Url2,
    target_arc_factor: u32,
) -> ConductorConfig {
    let network = NetworkConfig {
        bootstrap_url: bootstrap_url.to_owned(),
        signal_url: Url2::parse(webrtc_signal_url),
        relay_url: relay_url.to_owned(),
        target_arc_factor,
        ..NetworkConfig::default()
    };
    let mut config = ConductorConfig::default();
    config.db_sync_strategy = DbSyncStrategy::default();
    config.keystore = KeystoreConfig::LairServerInProc {
        lair_root: lair_path
            .as_ref()
            .map(|path_buf| KeystorePath::from(path_buf.clone())),
    };
    config.admin_interfaces = Some(vec![AdminInterfaceConfig {
        driver: InterfaceDriver::Websocket {
            port: admin_port,
            danger_bind_addr: None,
            allowed_origins: AllowedOrigins::Any,
        },
    }]);
    config.network = network;
    config.data_root_path = Some(DataRootPath::from(databases_path));
    config
}
