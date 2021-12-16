# deeper chain node

deeper chain node is built on top of Substrate v3.0.0 full node

## Local Development

Get the required compiler version and wasm component before compiling.

```
rustup install nightly-2021-10-21
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-10-21
rustup toolchain install nightly-2021-10-21

# fix environmental package bug if it happens
cargo update -p environmental

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

## Wallet Integration

See [this doc](wallet-integration.md)

## Update weights.rs in pallet
1. Build deeper-chain with `--features runtime-benchmarks`
```
cd cli/
cargo build --release --features runtime-benchmarks
```
2. Run shell command to update weights.rs
```
./target/release/deeper-chain benchmark  \
--chain=dev \
--steps=50 \
--repeat=20 \
--pallet=pallet_staking \
--extrinsic=* \
--execution=wasm \
--wasm-execution=compiled \
--heap-pages=4096 \
--output=./pallets/staking/src/weights.rs \
--template=./.maintain/frame-weight-template.hbs 
```