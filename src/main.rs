use embedded_runner::{async_main, HcConfig};
use emit::StateSignal;
use holochain::conductor::manager::handle_shutdown;
use holochain_trace::Output;
use read_passphrase_secure::{passphrase_from_bytes, SharedLockedArray};
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;
use url2::Url2;

mod config;
mod embedded_runner;
mod emit;
mod generate_key;
mod install_enable;
mod read_passphrase_secure;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "holochain-runner",
    about = "wrapped Holochain Conductor with Status Update events, and a good SIGTERM kill switch "
)]
struct Opt {
    #[structopt(help = "the path to a HAPP file to be
default installed to the app,
ending in .happ")]
    happ_path: PathBuf,

    #[structopt(
        default_value = "databases",
        help = "configuration values for `app_id` and `app_ws_port`
will be overridden if an existing
configuration is found at this path"
    )]
    datastore_path: PathBuf,

    #[structopt(long, default_value = "main-app")]
    app_id: String,

    #[structopt(long, default_value = "9090")]
    app_ws_port: u16,

    #[structopt(long, default_value = "1234", help = "")]
    admin_ws_port: u16,

    #[structopt(
        long,
        help = "This folder will store the private keys. It is encrypted on both Mac and Linux, but not Windows.
Per the behaviour of holochain itself, if you
do not pass a value here, it will use a default which is equal to the
value of `<datastore_path>/keystore`."
    )]
    keystore_path: Option<PathBuf>,

    #[structopt(
        long,
        default_value = "wss://dev-test-bootstrap2.holochain.org",
        help = "Websocket URL (wss) to a holochain kitsune2 signal server"
    )]
    webrtc_signal_url: String,

    #[structopt(
        long,
        parse(from_str = Url2::parse),
        default_value = "https://dev-test-bootstrap2.holochain.org",
        help = "URL of the kitsune2 bootstrap server"
    )]
    bootstrap_url: Url2,

    #[structopt(
        long,
        parse(from_str = Url2::parse),
        default_value = "https://use1-1.relay.n0.iroh-canary.iroh.link./",
        help = "URL of the iroh relay server"
    )]
    relay_url: Url2,

    #[structopt(long, help = "")]
    network_seed: Option<String>,

    #[structopt(
        long,
        default_value = "1",
        help = "The target arc factor to apply when receiving hints from kitsune2.
In normal operation, leave this as the default 1.
For leacher nodes that do not contribute to gossip, set to 0."
    )]
    target_arc_factor: u32,

    #[structopt(
        long,
        short,
        help = "Path to a local .env file containing a LAIR_PASSWORD variable"
    )]
    env_path: Option<PathBuf>,

    #[structopt(
        long,
        help = "Outputs structured json from logging:
    - None: No logging at all (fastest)
    - Log: Output logs to stdout with spans (human readable)
    - Compact: Same as Log but with less information
    - Json: Output logs as structured json (machine readable)
    ",
        default_value = "Log"
    )]
    logging: Output,
}

fn main() {
    // Create the runtime
    // we want to use multiple threads
    let rt = tokio::runtime::Builder::new_multi_thread()
        // we use both IO and Time tokio utilities
        .enable_all()
        // give our threads a descriptive name (they'll be numbered too)
        .thread_name("holochain-runner-tokio-thread")
        // build the runtime
        .build()
        // panic if we cannot (we cannot run without it)
        .expect("can build tokio runtime");
    let _guard = rt.enter();

    // set up the ctrlc shutdown listener
    // listening for SIGINT or SIGTERM (unix), just CTRC-C on windows
    let rt_handle = rt.handle().clone();

    // print each state signal to the terminal
    let (state_signal_sender, mut state_signal_receiver) =
        tokio::sync::mpsc::channel::<StateSignal>(10);
    tokio::task::spawn(async move {
        while let Some(signal) = state_signal_receiver.recv().await {
            println!("{}", state_signal_to_stdout(&signal));
        }
    });

    let opt = Opt::from_args();

    // Load .env file if provided

    let passphrase: SharedLockedArray = match opt.env_path {
        Some(path) => {
            println!("Looking for passphrase from env file");
            let env_val = dotenv::from_path(path.as_path())
                .unwrap_or_else(|_| panic!("Failed to parse env file from {path:?}"));
            env_val.load();
            let p = env::var("LAIR_PASSWORD").expect("No env var LAIR_PASSWORD found in env file");
            println!("Found passphrase, continuing...");

            passphrase_from_bytes(p.into_bytes())
        }
        _ => {
            println!("Looking for passphrase piped to stdin");
            let p = read_passphrase_secure::read_piped_passphrase()
                .expect("could not read piped passphrase");
            println!("Found passphrase, continuing...");

            p
        }
    };

    // An infinite stream of hangup signals.

    // Get a handle from this runtime
    tokio::task::block_in_place(|| {
        rt_handle.block_on(async {
            let conductor = async_main(
                passphrase,
                HcConfig {
                    app_id: opt.app_id,
                    happ_path: opt.happ_path,
                    admin_ws_port: opt.admin_ws_port,
                    app_ws_port: opt.app_ws_port,
                    datastore_path: opt.datastore_path,
                    keystore_path: opt.keystore_path,
                    webrtc_signal_url: opt.webrtc_signal_url,
                    event_channel: Some(state_signal_sender),
                    bootstrap_url: opt.bootstrap_url,
                    relay_url: opt.relay_url,
                    network_seed: opt.network_seed,
                    target_arc_factor: opt.target_arc_factor,
                    logging: opt.logging,
                },
            )
            .await;
            tokio::signal::ctrl_c().await.unwrap_or_else(|e| {
                tracing::error!("Could not handle termination signal: {:?}", e)
            });
            tracing::info!("Gracefully shutting down conductor...");
            let shutdown_result = conductor.shutdown().await;
            handle_shutdown(shutdown_result);
        })
    });
}

fn state_signal_to_stdout(signal: &StateSignal) -> i16 {
    match signal {
        StateSignal::IsFirstRun => 0,
        StateSignal::IsNotFirstRun => 1,
        // IsFirstRun events
        StateSignal::CreatingKeys => 2,
        StateSignal::RegisteringDna => 3,
        StateSignal::InstallingApp => 4,
        StateSignal::EnablingApp => 5,
        StateSignal::AddingAppInterface => 6,
        StateSignal::AuthenticatingAppInterface => 7,
        // Done/Ready Event
        StateSignal::IsReady => 8,
    }
}
