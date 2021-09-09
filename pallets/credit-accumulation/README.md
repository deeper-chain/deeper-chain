# Credit-Accumulation Pallet

The credit-accumulation pallet provides functionality for Deeper Connect devices to accumulate credit score by traffic.

## Overview

The credit-accumulation pallet provides the following functions:

- submit a signature to accumulate credit score
- set atmos pubkey


### Terminology

- **Nonce:** An index that indicates an occurring of sigature to accumulate credit score. It starts with 0 and increment by 1 each time.

## Interface

### Dispatchable Functions

- `add_credit_by_traffic` - an Atmos submit a signature to accumulate credit score.
- `set_atmos_pubkey` - root to set atmos pubkey.

## Usage

This pallet only provides dispatchable functions to end users.

Run `cargo build` in terminal to build this pallet.
Run `cargo test` in terminal to run the unit tests. 

## Genesis config

There is nothing to config for this pallet in genesis.

## Dependencies

This pallet depends on the CreditInterface trait. It's not a general purpose pallet that can be used elsewhere, but we hope the concepts and interfaces can be useful for other projects.

License: Apache-2.0
