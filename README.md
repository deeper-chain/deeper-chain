# deeper chain node

deeper chain node is built on top of Substrate v2.0.0 full node

## Local Development

Get the required compiler version and wasm component before compiling.

```
rustup install nightly-2020-10-06
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-06

# fix environmental package bug if it happens
cargo update -p environmental

# compile
cargo build --release
```

## Run

### Single Node Development Chain

Purge any existing dev chain state:

```bash
./target/release/e2-chain purge-chain --dev
```

Start a dev chain:

```bash
./target/release/e2-chain --dev
```

Or, start a dev chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/e2-chain -lruntime=debug --dev
```

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).
