use crate::{
    emit::{emit, StateSignal},
    generate_key::find_or_generate_key,
};
use holochain::conductor::{Conductor, ConductorHandle};
use holochain_client::{AdminWebsocket, IssueAppAuthenticationTokenPayload};
use holochain_trace::Output;
use holochain_types::prelude::{InstalledAppId, NetworkSeed};
use holochain_types::websocket::AllowedOrigins;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::*;
use url2::Url2;

pub type SharedLockedArray = Arc<Mutex<sodoken::LockedArray>>;

pub struct HcConfig {
    pub app_id: String,
    pub happ_path: PathBuf,
    pub admin_ws_port: u16,
    pub app_ws_port: u16,
    pub datastore_path: PathBuf,
    pub keystore_path: Option<PathBuf>,
    pub webrtc_signal_url: String,
    pub event_channel: Option<mpsc::Sender<StateSignal>>,
    pub bootstrap_url: Url2,
    pub relay_url: Url2,
    pub network_seed: Option<NetworkSeed>,
    pub target_arc_factor: u32,
    pub logging: Output,
}

pub async fn async_main(passphrase: SharedLockedArray, hc_config: HcConfig) -> ConductorHandle {
    // Sets up a human-readable panic message with a request for bug reports
    // See https://docs.rs/human-panic/1.0.3/human_panic/
    human_panic::setup_panic!();
    // take in command line arguments
    holochain_trace::init_fmt(hc_config.logging).expect("Failed to start contextual logging");
    debug!("holochain_trace initialized");
    if !hc_config.datastore_path.as_path().exists() {
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
        hc_config.datastore_path.clone(),
        &hc_config.keystore_path,
        &hc_config.webrtc_signal_url,
        &hc_config.bootstrap_url,
        &hc_config.relay_url,
        hc_config.target_arc_factor,
    )
    .await;

    println!(
        "DATASTORE_PATH: {}",
        hc_config.datastore_path.as_path().display()
    );
    println!("KEYSTORE_PATH: {:?}", hc_config.keystore_path);
    println!("NETWORK_SEED: {:?}", hc_config.network_seed);

    // install the app with its dnas, if they aren't already
    // as well as adding the app_ws_port
    let conductor_copy = conductor.clone();
    let _handle = tokio::task::spawn(async move {
        match install_or_passthrough(
            &conductor_copy,
            hc_config.app_id,
            hc_config.admin_ws_port,
            hc_config.app_ws_port,
            hc_config.happ_path,
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
    passphrase: SharedLockedArray,
    admin_ws_port: u16,
    databases_path: PathBuf,
    keystore_path: &Option<PathBuf>,
    webrtc_signal_url: &str,
    bootstrap_url: &Url2,
    relay_url: &Url2,
    target_arc_factor: u32,
) -> ConductorHandle {
    let config = super::config::conductor_config(
        admin_ws_port,
        databases_path,
        keystore_path,
        webrtc_signal_url,
        bootstrap_url,
        relay_url,
        target_arc_factor,
    );
    // Initialize the Conductor
    Conductor::builder()
        .config(config)
        .passphrase(Some(passphrase))
        .build()
        .await
        .expect("Could not initialize Conductor from configuration")
}

async fn install_or_passthrough(
    conductor: &ConductorHandle,
    app_id: InstalledAppId,
    admin_ws_port: u16,
    app_ws_port: u16,
    happ_path: PathBuf,
    event_channel: &Option<mpsc::Sender<StateSignal>>,
    network_seed: Option<NetworkSeed>,
) -> anyhow::Result<()> {
    let admin_client =
        AdminWebsocket::connect(format!("localhost:{}", admin_ws_port), None).await?;

    let app_infos = admin_client
        .list_apps(None)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list apps {:?}", e))?;
    let matching_index = app_infos
        .iter()
        .position(|info| info.installed_app_id == app_id);

    match matching_index {
        Some(_) => {
            println!("An existing app was found with this app_id. Skipping app install.");
        }
        _ => {
            // Install App
            println!("There is no app installed, so starting fresh...");
            super::install_enable::install_app(
                &admin_client,
                find_or_generate_key(conductor, event_channel).await?,
                app_id.clone(),
                happ_path,
                event_channel,
                network_seed,
            )
            .await?;

            // Enable App
            println!("Installed, now enabling...");
            super::install_enable::enable_app(&admin_client, app_id.clone(), event_channel).await?;
        }
    };

    let app_interface_infos = admin_client
        .list_app_interfaces()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list app interfaces {:?}", e))?;
    println!("app interface infos {:?}", app_interface_infos);
    let matching_index = app_interface_infos.iter().position(|info| {
        info.installed_app_id.clone() == Some(app_id.clone()) && info.port == app_ws_port
    });

    let app_ws_token: Option<Vec<u8>> = match matching_index {
        Some(_) => {
            println!("An existing app interface was found with this app_id and app_ws_port. Skipping creating app websocket.");
            None
        }
        _ => {
            // Create App Websocket Interface
            emit(event_channel, StateSignal::AddingAppInterface).await;
            println!("Enabled, now creating app websocket...");
            admin_client
                .attach_app_interface(app_ws_port, None, AllowedOrigins::Any, Some(app_id.clone()))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to attach app interface {:?}", e))?;

            // Issue authentication token for app websocket inferface
            emit(event_channel, StateSignal::AuthenticatingAppInterface).await;
            println!("Created, now issuing authentication token for app websocket...");
            let app_auth = admin_client
                .issue_app_auth_token(IssueAppAuthenticationTokenPayload {
                    installed_app_id: app_id.clone(),
                    expiry_seconds: u64::MAX,
                    single_use: false,
                })
                .await
                .map_err(|e| anyhow::anyhow!("Failed to attach app interface {:?}", e))?;
            println!("Issued.");

            Some(app_auth.token)
        }
    };
    emit(event_channel, StateSignal::IsReady).await;

    println!("APP_WS_PORT: {}", app_ws_port);
    if let Some(token) = app_ws_token {
        println!(
            "APP_WS_TOKEN: !!! THIS WILL ONLY BE SHOWN ONCE !!!\n{:?}",
            token
        );
    }
    println!("INSTALLED_APP_ID: {}", app_id);
    println!("HOLOCHAIN_RUNNER_IS_READY");

    Ok(())
}
