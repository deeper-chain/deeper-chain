# Deeper-chain: A Comprehensive Guide
![Build](https://github.com/deeper-chain/deeper-chain/actions/workflows/build.yml/badge.svg)
[![Codecov](https://codecov.io/gh/deeper-chain/deeper-chain/branch/master/graph/badge.svg)](https://codecov.io/gh/deeper-chain/deeper-chain)
## 1. Introduction

### Background

Deeper-chain is a groundbreaking blockchain platform that aims to redefine the landscape of decentralized networks. Built on the robust Substrate framework, it offers a secure, scalable, and efficient environment for decentralized applications (dApps) and smart contracts. This platform is not just another blockchain; it is the cornerstone of a new era in blockchain technology. It brings a plethora of features that are geared towards enhancing user experience, developer flexibility, and overall network performance.

### Objectives

The primary objectives of Deeper-chain are multi-faceted:

1. **Security**: To provide a secure and tamper-proof system that can withstand various types of attacks.
2. **High Throughput**: To ensure high throughput so that transactions are processed quickly.
3. **Low Latency**: To maintain low latency for real-time applications.
4. **Interoperability**: To facilitate seamless interoperability between different blockchain networks.
5. **Community Governance**: To offer a robust governance model that allows for community-driven updates and changes.

### Scope

This document serves as a comprehensive guide for developers, node operators, and community members. It aims to be the go-to resource for anyone interested in understanding, installing, and contributing to Deeper-chain. It covers everything from the basic features to advanced modules and integration points.

### Terminology

- **Node**: A computer connected to the Deeper-chain network that participates in network activities.
- **Validator**: A specialized node that participates in the consensus algorithm to validate transactions and create new blocks.
- **Nominator**: A token holder who nominates a validator to act on their behalf.
- **Governance**: The system by which decisions are made within the network, often involving proposals and voting mechanisms.

## 2. Features of Deeper-chain

### 2.1 Security Measures

Deeper-chain employs a multi-layered security architecture that includes advanced cryptographic techniques, a robust consensus algorithm, and additional mechanisms to guard against malicious activities. Features such as Runtime Verification and Off-chain Workers contribute to the overall security of the network. The platform also employs various other security measures like two-factor authentication, cold storage options for assets, and regular security audits.

### 2.2 Scalability Solutions

Deeper-chain addresses the challenges of scalability by employing a range of solutions. It utilizes sharding technology alongside parallel transaction processing, which ensures that the network can handle a large number of transactions without compromising on speed or efficiency. The platform also employs various layer-2 solutions like state channels and rollups to further enhance its scalability.

### 2.3 Efficiency Metrics

Deeper-chain is designed to be an energy-efficient blockchain. It employs mechanisms like Proof-of-Stake (PoS) and other consensus algorithms that require less computational power, thereby reducing the overall energy consumption. The platform also focuses on optimizing code and reducing computational complexity wherever possible.

### 2.4 Interoperability

Interoperability is one of the standout features of Deeper-chain. Built on the Substrate framework, it is inherently compatible with other blockchains in the Polkadot ecosystem. This allows for seamless asset and data transfer across different networks. The platform also supports various cross-chain communication protocols to facilitate interaction with blockchains outside the Polkadot ecosystem.

### 2.5 Governance Model

Deeper-chain adopts a community-driven governance model. It empowers its users to propose changes and vote on them. This democratic approach ensures that the network evolves in a way that benefits its entire ecosystem. The governance model is designed to be transparent, inclusive, and fair, allowing for a wide range of proposals to be considered and implemented.

## 3. Prerequisites

### 3.1 Software Requirements

- **Rust v1.50.0 or above**: The Rust programming language is essential for compiling and running the Deeper-chain node.
- **Substrate v2.0.0 or above**: Substrate is the framework upon which Deeper-chain is built. It provides the basic building blocks for creating a blockchain.
- **Node.js v14.0.0 or above**: Node.js is required for running various scripts and tasks.
- **Yarn package manager**: Yarn is used for managing Node.js packages.

### 3.2 Hardware Requirements

- **A minimum of 4 GB RAM**: Adequate memory is essential for smooth operation.
- **At least 50 GB of free disk space**: Sufficient storage is necessary for holding the blockchain data.
- **A stable internet connection**: A reliable internet connection is crucial for staying synced with the network.

## 4. Installation and Setup

### 4.1 Environment Configuration

Before you begin the installation process, you need to set up the environment variables. This involves configuring the Rust toolchain, setting up the PATH variables, and installing the required Node.js packages.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME

/.cargo/env
```

### 4.2 Building from Source

To build Deeper-chain from source, you'll need to clone the repository and compile the code. This can be done using the following commands:

```bash
git clone https://github.com/Deeper-chain/Deeper-chain.git
cd Deeper-chain
cargo build --release
```

### 4.3 Running a Node

After successfully building from source, you can run a Deeper-chain node using the following command:

```bash
./target/release/deeper-chain --dev
```

### 4.4 Single Node Development Chain

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

### 4.5 Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

This will start a development node on your local machine. For running a node in a production environment, additional flags and configurations are required.


## 5. Update weights.rs in pallet
### 5.1 Build deeper-chain with `--features runtime-benchmarks`
```
cd cli/
cargo build --release --features runtime-benchmarks
```
### 5.2 Run shell command to update weights.rs
```
./target/release/deeper-chain benchmark pallet \
--chain=dev \
--steps=50 \
--repeat=20 \
--pallet=pallet_staking \
--extrinsic='*' \
--execution=wasm \
--wasm-execution=compiled \
--heap-pages=4096 \
--output=./pallets/staking/src/weights.rs \
--template=./.maintain/frame-weight-template.hbs 
```


## 6. Network Architecture

### 6.1 Node Types

Deeper-chain supports various types of nodes, each serving a specific function within the network:

1. **Full Nodes**: These nodes store the entire blockchain data and validate all transactions. They are the backbone of the network, ensuring data integrity and serving as a relay for transactions.
   
2. **Validator Nodes**: These are specialized full nodes that participate in the consensus algorithm. They validate transactions, create new blocks, and are responsible for the overall security of the network. Validators are chosen based on a combination of factors including stake, uptime, and reputation.

3. **Light Nodes**: These nodes do not store the entire blockchain but can still validate transactions and blocks. They are optimized for low-resource environments and are ideal for mobile applications and IoT devices.

### 6.2 Consensus Mechanism

Deeper-chain employs a hybrid consensus mechanism that combines elements of Proof-of-Stake (PoS) and Byzantine Fault Tolerance (BFT). This ensures a high level of security while maintaining efficiency and scalability. Validators are chosen based on the amount of stake they hold, and they participate in the consensus process to validate transactions and produce new blocks.

### 6.3 Network Topology

The network architecture of Deeper-chain is designed to be modular and flexible. It consists of various layers, including the core blockchain layer, the consensus layer, and the application layer. Each layer is designed to be interchangeable, allowing for easy upgrades and modifications.

## 7. Smart Contracts and dApps

### 7.1 Smart Contract Development

Deeper-chain provides a rich environment for developing smart contracts. It supports multiple programming languages, including Solidity and Rust, and offers a range of pre-built modules for common functionalities. Developers can also make use of the extensive documentation and tutorials available to get started with smart contract development.

### 7.2 DApp Ecosystem

Deeper-chain supports dApps in various sectors including finance, advertising, oracles, and the Internet. The platform provides a rich set of tools and SDKs to facilitate dApp development. With its robust features and high-performance metrics, Deeper-chain aims to be the go-to platform for dApp development. It offers various tools and SDKs to facilitate the development process, making it easier for developers to bring their ideas to life.

## 8. Wallet Integration

### 8.1 Web Wallet

Deeper-chain supports web-based wallets that offer a user-friendly interface for managing your assets. These wallets are secured with advanced cryptographic techniques to ensure the safety of your funds. They also offer features like multi-signature support, transaction history, and more.

### 8.2 Hardware Wallet

Deeper-chain has developed its own hardware wallet, designed to securely store various digital assets. These physical devices offer an extra layer of protection, safeguarding your assets from online threats. They are compatible with most major hardware wallet brands and offer a straightforward setup process.

## 9. Development Guidelines

### 9.1 Code Review Process

All code contributions to Deeper-chain go through a rigorous review process. This involves automated testing, manual review by core developers, and a final approval from the governance committee. The platform employs Continuous Integration (CI) and Continuous Deployment (CD) practices to ensure that all code changes are automatically tested and deployed.

### 9.2 Benchmarking and Weight Calculation

Performance is a key focus in Deeper-chain development. All new features and updates must undergo benchmarking to assess their impact on network performance. Weight calculations are also performed to ensure that the network remains balanced and efficient. Various performance metrics are monitored, including transaction speed, block time, and resource utilization.

## 10. Contributing to Deeper-chain

### 10.1 Code Contributions

Developers are encouraged to contribute to Deeper-chain by submitting pull requests. Detailed guidelines for code contributions can be found in the `CONTRIBUTING.md` file. The platform also offers a bug bounty program to incentivize the discovery and reporting of security vulnerabilities.

### 10.2 Documentation Updates

Updates to documentation are equally important. Whether it's fixing typos or adding new sections, all contributions are welcome. The platform maintains a separate repository for documentation, and contributions can be made through pull requests.

### 10.3 Community Engagement

Community involvement is crucial for the growth and success of Deeper-chain. Users can participate in governance, propose new features, and engage in discussions to shape the future of the network. Various community channels like forums, social media, and chat rooms are available for users to connect and collaborate.

## 11. License and Compliance

Deeper-chain is released under the MIT License, ensuring that it remains open-source and accessible to all. Compliance with legal and regulatory standards is of utmost importance, and users are advised to read the `LICENSE.md` file for more details. The platform also undergoes regular audits to ensure compliance with data protection and financial regulations.