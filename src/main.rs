use embedded_runner::{async_main, HcConfig};
use emit::StateSignal;
use holochain_p2p::kitsune_p2p::dependencies::url2::Url2;
use std::path::PathBuf;
use structopt::StructOpt;

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
    datastore_path: String,

    #[structopt(long, default_value = "main-app")]
    app_id: String,

    #[structopt(long)]
    app_ws_port: Option<u16>,

    #[structopt(long, default_value = "1234")]
    admin_ws_port: u16,

    #[structopt(long, parse(from_str = Url2::parse))]
    keystore_url: Url2,

    // #[structopt(long)]
    // membrane_proof: Option<String>,

    // community
    #[structopt(
        long,
        default_value = "kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--"
    )]
    proxy_url: String,

    #[structopt(long, parse(from_str = Url2::parse))]
    bootstrap_url: Option<Url2>,

    #[structopt(long)]
    network_seed: Option<String>,
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
    let (shutdown_sender, mut shutdown_receiver) = tokio::sync::mpsc::channel::<bool>(1);
    ctrlc::set_handler(move || {
        println!("ctrlc got shutdown signal");
        // send shutdown signal
        let shutdown_sender_c = shutdown_sender.clone();
        tokio::task::block_in_place(|| {
            rt.block_on(async {
                // trigger shutdown
                match shutdown_sender_c.send(true).await {
                    Ok(_) => {}
                    Err(_) => {}
                };
            });
        });
    })
    .expect("Error setting Ctrl-C handler");

    // print each state signal to the terminal
    let (state_signal_sender, mut state_signal_receiver) =
        tokio::sync::mpsc::channel::<StateSignal>(10);
    tokio::task::spawn(async move {
        while let Some(signal) = state_signal_receiver.recv().await {
            println!("{}", state_signal_to_stdout(&signal));
        }
    });

    let opt = Opt::from_args();

    // An infinite stream of hangup signals.

    // Get a handle from this runtime
    tokio::task::block_in_place(|| {
        rt_handle.block_on(async {
            println!("Looking for passphrase piped to stdin");
            let passphrase = read_passphrase_secure::read_piped_passphrase()
            .await
            .unwrap();
            println!("Found passphrase, continuing...");

            let shutdown_oneshot_sender = async_main(
                passphrase,
                HcConfig {
                    app_id: opt.app_id,
                    happ_path: opt.happ_path,
                    admin_ws_port: opt.admin_ws_port,
                    app_ws_port: opt.app_ws_port,
                    datastore_path: opt.datastore_path,
                    keystore_url: opt.keystore_url,
                    proxy_url: opt.proxy_url,
                    // membrane_proof: opt.membrane_proof,
                    event_channel: Some(state_signal_sender),
                    bootstrap_url: opt.bootstrap_url,
                    network_seed: opt.network_seed,
                },
            )
            .await;
            // wait for shutdown signal
            shutdown_receiver.recv().await;
            // pass that signal through to embedded-holochain-runner
            match shutdown_oneshot_sender.send(true) {
                Ok(()) => {
                    println!("successfully sent shutdown signal to embedded-holochain-runner");
                }
                Err(_) => {
                    println!("the receiver of the oneshot sender must have been dropped");
                    panic!()
                }
            };
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
