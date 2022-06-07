# holochain-runner

> Holochain Revision: [v0.0.141  May 24, 2022](https://github.com/holochain/holochain/releases/tag/holochain-0.0.141)
> 
> Lair Keystore Revision: [v0.0.10 Apr 25, 2022](https://github.com/holochain/lair/releases/tag/v0.0.10)
>
> Expects an HAPP built with HDK [v0.0.134](https://docs.rs/hdk/0.0.134/hdk/index.html)

An alternative Holochain conductor binary useful for quick startup and inclusive handling of key generation and dna installation
for a multi-DNA hApp.

```bash
holochain-runner 0.0.38
wrapped Holochain Conductor with Status Update events, and a good SIGTERM kill switch 

USAGE:
    holochain-runner [OPTIONS] <happ-path> [datastore-path]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --admin-ws-port <admin-ws-port>     [default: 1234]
        --app-id <app-id>                   [default: main-app]
        --app-ws-port <app-ws-port>        
        --bootstrap-url <bootstrap-url>    
        --keystore-path <keystore-path>     [default: keystore]
        --proxy-url <proxy-url>             [default: kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-
                                           qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--]
        --uid <uid>                        

ARGS:
    <happ-path>         the path to a HAPP file to be
                        default installed to the app,
                        ending in .happ
    <datastore-path>    configuration values for `app_id` and `app_ws_port`
                        will be overridden if an existing
                        configuration is found at this path [default: databases]
```
## How it will work

`datastore-path` is most important. If existing persisted Holochain conductor files
are found in the given directory, it will simply re-use the `admin_ws_port` `app_ws_port` `app_id` and `dnas` from that configuration. Otherwise, it will create that directory, and setup your configuration as specified.

`keystore-path` can point to an empty folder, or a pre-existing keystore, as long as that keystore uses a compatible keystore format. If there is a private key in the existing keystore it will use that to install the HAPP, if there is none, it will generate one automatically on the first run.

It uses structopt to make a configurable service. For a more advanced application using shutdown signal, and `StateSignal` listeners, you can see it in use in the [Acorn Holochain application](https://github.com/h-be/acorn/blob/main/conductor/src/main.rs).

In either case,

- first run/installation
- second run/reboot

it will log this to the console when the interfaces are all ready and the app installed or running:

`EMBEDDED_HOLOCHAIN_IS_READY`

It will clearly log its configuration to the console.

RUST_LOG environment variable can be set to get details logs from Holochain. Those logs are by default suppressed.

## Events

It may emit events, based on event types in an enum `StateSignal`. These will be logged to the console
so that you can track the internal state and progress.

It looks like:

```rust
pub enum StateSignal {
    // will be only one or the other of these
    IsFirstRun,
    IsNotFirstRun,
    // are sub events after IsFirstRun
    CreatingKeys,
    RegisteringDna,
    InstallingApp,
    EnablingApp,
    AddingAppInterface,
    // Done/Ready Event, called when websocket interfaces and
    // everything else is ready
    IsReady,
}
```

## Bootstrap Networking Service

This library is currently by default pointed at the `https://bootstrap-staging.holo.host` node discovery service, but can be overridden.
