use holochain::conductor::config::{
    AdminInterfaceConfig, ConductorConfig, InterfaceDriver, KeystoreConfig,
};
use holochain_p2p::kitsune_p2p::{
    dependencies::url2::{self, Url2},
    KitsuneP2pConfig, ProxyConfig, TransportConfig,
};
use holochain_types::env::DbSyncStrategy;
use std::path::PathBuf;

pub fn conductor_config(
    admin_port: u16,
    databases_path: &str,
    keystore_path: &str,
    proxy_url: &str,
    maybe_boostrap_url: Option<Url2>,
) -> ConductorConfig {
    // Build the conductor configuration
    let mut network_config = KitsuneP2pConfig::default();
    network_config.bootstrap_service = match maybe_boostrap_url {
        None => Some(url2::url2!("https://bootstrap-staging.holo.host")),
        Some(url) => Some(url),
    };
    network_config.transport_pool.push(TransportConfig::Proxy {
        sub_transport: Box::new(TransportConfig::Quic {
            bind_to: Some(url2::url2!("kitsune-quic://0.0.0.0:0")),
            override_host: None,
            override_port: None,
        }),
        proxy_config: ProxyConfig::RemoteProxyClient {
            proxy_url: Url2::parse(proxy_url),
        },
    });
    ConductorConfig {
        environment_path: PathBuf::from(databases_path).into(),
        dpki: None,
        db_sync_strategy: DbSyncStrategy::default(),
        keystore: KeystoreConfig::LairServerLegacyDeprecated {
            keystore_path: Some(PathBuf::from(keystore_path)),
            danger_passphrase_insecure_from_config: (String::from("passphrase-placeholder")),
        },
        admin_interfaces: Some(vec![AdminInterfaceConfig {
            driver: InterfaceDriver::Websocket { port: admin_port },
        }]),
        network: Some(network_config),
    }
}
