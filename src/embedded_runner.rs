use crate::{
    emit::{emit, StateSignal},
    generate_key::find_or_generate_key,
};
use either::*;
use holochain::conductor::{api::error::ConductorApiResult, Conductor, ConductorHandle};
use holochain_p2p::kitsune_p2p::dependencies::url2::Url2;
use holochain_types::app::InstalledAppId;
use holochain_zome_types::NetworkSeed;
use observability::Output;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::*;

pub struct HcConfig {
    pub app_id: String,
    pub happ_path: PathBuf,
    pub admin_ws_port: u16,
    pub app_ws_port: u16,
    pub datastore_path: String,
    pub keystore_path: Option<PathBuf>,
    // pub membrane_proof: Option<String>,
    pub webrtc_signal_url: String,
    pub event_channel: Option<mpsc::Sender<StateSignal>>,
    pub bootstrap_url: Url2,
    pub network_seed: Option<NetworkSeed>,
    pub gossip_arc_clamping: String,
}

pub async fn async_main(passphrase: sodoken::BufRead, hc_config: HcConfig) -> ConductorHandle {
    // Sets up a human-readable panic message with a request for bug reports
    // See https://docs.rs/human-panic/1.0.3/human_panic/
    human_panic::setup_panic!();
    // take in command line arguments
    observability::init_fmt(Output::Log).expect("Failed to start contextual logging");
    debug!("observability initialized");
    // Uncomment this to get regular networking info status updates in the logs
    // kitsune_p2p_types::metrics::init_sys_info_poll();
    if !Path::new(&hc_config.datastore_path).exists() {
        emit(&hc_config.event_channel, StateSignal::IsFirstRun).await;
        if let Err(e) = std::fs::create_dir(&hc_config.datastore_path) {
            error!("{}", e);
            panic!()
        };
    } else {
        emit(&hc_config.event_channel, StateSignal::IsNotFirstRun).await;
    }
    // run up a conductor
    let conductor = conductor_handle(
        passphrase,
        hc_config.admin_ws_port,
        &hc_config.datastore_path,
        &hc_config.keystore_path,
        &hc_config.webrtc_signal_url,
        &hc_config.bootstrap_url,
        &hc_config.gossip_arc_clamping,
    )
    .await;

    println!("DATASTORE_PATH: {}", hc_config.datastore_path);
    println!("KEYSTORE_PATH: {:?}", hc_config.keystore_path);
    println!("NETWORK_SEED: {:?}", hc_config.network_seed);

    // install the app with its dnas, if they aren't already
    // as well as adding the app_ws_port
    let conductor_copy = conductor.clone();
    let _handle = tokio::task::spawn(async move {
        match install_or_passthrough(
            &conductor_copy,
            hc_config.app_id,
            hc_config.app_ws_port,
            hc_config.happ_path,
            // hc_config.membrane_proof,
            &hc_config.event_channel,
            hc_config.network_seed,
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("{}", e);
                panic!()
            }
        }
    });

    conductor
}

async fn conductor_handle(
    passphrase: sodoken::BufRead,
    admin_ws_port: u16,
    databases_path: &str,
    keystore_path: &Option<PathBuf>,
    webrtc_signal_url: &str,
    bootstrap_url: &Url2,
    gossip_arc_clamping: &str,
) -> ConductorHandle {
    let config = super::config::conductor_config(
        admin_ws_port,
        databases_path,
        keystore_path,
        webrtc_signal_url,
        bootstrap_url,
        gossip_arc_clamping,
    );
    // Initialize the Conductor
    Conductor::builder()
        .config(config)
        .passphrase(Some(passphrase))
        .build()
        .await
        .expect("Could not initialize Conductor from configuration")
}

#[allow(deprecated)]
async fn install_or_passthrough(
    conductor: &ConductorHandle,
    app_id: InstalledAppId,
    app_ws_port: u16,
    happ_path: PathBuf,
    // membrane_proof: Option<String>,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
    network_seed: Option<NetworkSeed>,
) -> ConductorApiResult<()> {
    let app_ids = conductor.list_apps(None).await?;
    // defaults
    let using_app_ws_port: u16;

    let agent_key = find_or_generate_key(conductor, event_channel).await?;

    if app_ids.is_empty() {
        println!("There is no app installed, so starting fresh...");
        super::install_enable::install_app(
            conductor,
            agent_key,
            app_id.clone(),
            happ_path,
            // membrane_proof,
            event_channel,
            network_seed,
        )
        .await?;
        println!("Installed, now enabling...");
        super::install_enable::enable_app(conductor, app_id.clone(), event_channel).await?;
        // add a websocket interface on the first run
        // it will boot again at the same interface on second run
        emit(event_channel, StateSignal::AddingAppInterface).await;
        using_app_ws_port = conductor
            .clone()
            .add_app_interface(Either::Left(app_ws_port))
            .await?;
        println!("Enabled.");
    } else {
        println!("An existing configuration and identity was found, using that.");
        let app_ports = conductor.list_app_interfaces().await?;
        if !app_ports.is_empty() {
            using_app_ws_port = app_ports[0];
        } else {
            println!("No app port is attached, adding one.");
            using_app_ws_port = conductor
                .clone()
                .add_app_interface(Either::Left(app_ws_port))
                .await?;
        }
    }

    emit(event_channel, StateSignal::IsReady).await;
    println!("     APP_WS_PORT: {}", using_app_ws_port);
    println!("INSTALLED_APP_ID: {}", app_id);
    println!("HOLOCHAIN_RUNNER_IS_READY");
    Ok(())
}
