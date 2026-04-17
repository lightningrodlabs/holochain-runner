# holochain-runner

<!-- > Underlying Holochain Version: [v0.6.1-rc.7](https://github.com/holochain/holochain/blob/main-0.6/CHANGELOG.md) -->
<!-- > -->
<!-- > Expects an HAPP built with HDK and HDI compatible with Holochain 0.6 -->

An alternative Holochain conductor binary useful for quick startup and including handling of key generation and hApp installation.

```bash
holochain-runner 0.11.0
wrapped Holochain Conductor with Status Update events, and a good SIGTERM kill switch

USAGE:
    holochain-runner [OPTIONS] <happ-path> [datastore-path]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --admin-ws-port <admin-ws-port>                 [default: 1234]
        --app-id <app-id>                               [default: main-app]
        --app-ws-port <app-ws-port>                     [default: 9090]
        --bootstrap-url <bootstrap-url>
            URL of the kitsune2 bootstrap server [default: https://dev-test-bootstrap2.holochain.org]
    -e, --env-path <env-path>                          Path to a local .env file containing a LAIR_PASSWORD variable
        --keystore-path <keystore-path>
            This folder will store the private keys. It is encrypted on both Mac and Linux, but not Windows.
            Per the behaviour of holochain itself, if you
            do not pass a value here, it will use a default which is equal to the
            value of `<datastore_path>/keystore`.
        --logging <logging>
            Outputs structured json from logging:
                - None: No logging at all (fastest)
                - Log: Output logs to stdout with spans (human readable)
                - Compact: Same as Log but with less information
                - Json: Output logs as structured json (machine readable)
                 [default: Log]
        --network-seed <network-seed>
        --relay-url <relay-url>
            URL of the iroh relay server [default: https://use1-1.relay.n0.iroh-canary.iroh.link./]
        --target-arc-factor <target-arc-factor>
            The target arc factor to apply when receiving hints from kitsune2.
            In normal operation, leave this as the default 1.
            For leacher nodes that do not contribute to gossip, set to 0. [default: 1]
        --webrtc-signal-url <webrtc-signal-url>
            Websocket URL (wss) to a holochain kitsune2 signal server [default: wss://dev-test-bootstrap2.holochain.org]


ARGS:
    <happ-path>         the path to a HAPP file to be
                        default installed to the app,
                        ending in .happ
    <datastore-path>    configuration values for `app_id` and `app_ws_port`
                        will be overridden if an existing
                        configuration is found at this path [default: databases]
```

## How it will work

Lair Keystore setup requires use of a passphrase for encryption and security.
You should pipe the passphrase to `holochain-runner` as STDIN, so that it can unlock the lair-keystore and connect to it. You do not need to pass `-p`, it assumes the password will be piped.

`datastore-path` is most important. If existing persisted Holochain conductor files
are found in the given directory, it will simply re-use the `admin_ws_port` `app_ws_port` `app_id` and `dnas` from that configuration. Otherwise, it will create that directory, and setup your configuration as specified.

`keystore-path` can point to an empty folder, or a pre-existing keystore, as long as that keystore uses a compatible keystore format. If there is a private key in the existing keystore it will use that to install the HAPP, if there is none, it will generate one automatically on the first run.

It uses structopt to make a configurable service. For a more advanced application using shutdown signal, and `StateSignal` listeners, you can see it in use in the [Acorn Holochain application](https://github.com/h-be/acorn/blob/main/conductor/src/main.rs).

In either case,

- first run/installation
- second run/reboot

it will log this to the console when the interfaces are all ready and the app installed or running:

`HOLOCHAIN_RUNNER_IS_READY`

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
    AuthenticatingAppInterface,
    // Done/Ready Event, called when websocket interfaces and
    // everything else is ready
    IsReady,
}
```

## Bootstrap Networking Service

This library is currently by default pointed at the `https://dev-test-bootstrap2.holochain.org` kitsune2 bootstrap service, but can be overridden.

## Signal Service

This library is currently by default pointed at the `wss://dev-test-bootstrap2.holochain.org` kitsune2 signal service, but can be overridden.

## Relay Service

This library is currently by default pointed at the `https://use1-1.relay.n0.iroh-canary.iroh.link./` iroh relay service, but can be overridden.
