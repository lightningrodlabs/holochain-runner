# holochain-runner

> Holochain Revision: [0.0.115 Nov 10, 2021](https://github.com/holochain/holochain/releases/tag/holochain-0.0.115)

An alternative Holochain conductor binary useful for quick startup and inclusive handling of key generation and dna installation
for a single DNA app.

```bash
holochain-runner 0.0.30
wrapped Holochain Conductor with Status Update events and a good SIGTERM kill switch.

USAGE:
    holochain-runner [OPTIONS] <dna-path> [datastore-path]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --admin-ws-port <admin-ws-port>     [default: 1234]
        --app-id <app-id>                   [default: main-app]
        --app-ws-port <app-ws-port>         [default: 8888]
        --keystore-path <keystore-path>     [default: keystore]
        --membrane-proof <membrane-proof>   (optional) -> base64 encoded string
        --proxy-url <proxy-url>             [default: kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-
                                           qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--]

ARGS:
    <dna-path>          the path to a DNA file to be
                        default installed to the app,
                        ending in .dna
    <datastore-path>    configuration values for `app_id` and `app_ws_port`
                        will be overridden if an existing
                        configuration is found at this path [default: databases]
```
