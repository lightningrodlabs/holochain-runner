use embedded_runner::{async_main, HcConfig};
use emit::StateSignal;
use holochain::conductor::manager::handle_shutdown;
use holochain_p2p::kitsune_p2p::dependencies::url2::Url2;
use std::path::PathBuf;
use structopt::StructOpt;
use dotenv;
use std::env;

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

    // the 0 default here will just let the
    // system pick a port
    #[structopt(
        long,
        default_value = "0",
        help = "The 0 default value here really means that
a random open port will be selected if you don't pass one.
The selected value will be reported out in the logs."
    )]
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

    // #[structopt(long)]
    // membrane_proof: Option<String>,

    // community
    #[structopt(
        long,
        parse(from_str = Url2::parse),
        default_value = "kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--",
        help = ""
    )]
    proxy_url: Url2,

    #[structopt(
        long,
        parse(from_str = Url2::parse),
        default_value = "https://bootstrap.holo.host",
        help = ""
    )]
    bootstrap_url: Url2,

    #[structopt(long, help = "")]
    network_seed: Option<String>,

    #[structopt(
        long,
        short,
        help = "Path to a local .env file containing a LAIR_PASSWORD variable"
    )]
    env_path: Option<PathBuf>,
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
    
    let passphrase: sodoken::BufRead = match opt.env_path {
        Some(path) => {
            println!("Looking for passphrase from env file");
            let env_val = dotenv::from_path(path.as_path())
                .expect(format!("Failed to parse env file from {path:?}").as_str());
            env_val.load();
            let p = env::var("LAIR_PASSWORD").expect("No env var LAIR_PASSWORD found in env file");
            println!("Found passphrase, continuing...");

            p.as_bytes().to_vec().into()
        },
        None => {
            println!("Looking for passphrase piped to stdin");
            let p: sodoken::BufRead = read_passphrase_secure::read_piped_passphrase()
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
                    proxy_url: opt.proxy_url,
                    // membrane_proof: opt.membrane_proof,
                    event_channel: Some(state_signal_sender),
                    bootstrap_url: opt.bootstrap_url,
                    network_seed: opt.network_seed,
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
        // Done/Ready Event
        StateSignal::IsReady => 7,
    }
}
