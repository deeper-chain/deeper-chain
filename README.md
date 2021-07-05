# deeper chain node

deeper chain node is built on top of Substrate v2.0.0 full node

## Local Development

Get the required compiler version and wasm component before compiling.

```
rustup install nightly-2021-03-11
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-11

# fix environmental package bug if it happens
cargo update -p environmental

# thread_local 1.1.2 has a bug: "memory leak"
cargo update -p thread_local

# compile
cargo build --release
```

## Run

### Single Node Development Chain

Purge any existing dev chain state:

```bash
./target/release/deeper-chain purge-chain --dev
```

Start a dev chain:

```bash
./target/release/deeper-chain --dev
```

Or, start a dev chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/deeper-chain -lruntime=debug --dev
```

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).
