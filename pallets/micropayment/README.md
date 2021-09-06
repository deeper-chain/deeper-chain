# Micropayment Pallet

The Micropayment pallet provides functionality for Deeper Connect devices to open/close micorpayment channels and for servicing device to claim micropayments from the channel.

## Overview

The Micropayment pallet provides the following functions:

For client
- Open a micropayment channel
- Add balance to the existing channel
- Close an expired channel
- Close all the expired channels

For server
- Close a micropayment channel
- Claim payment from the micropayment channel

### Terminology

- **Client:** A client is a Deeper Connect device that requests data proxy service from another Deeper Connect device. Any Deeper Connect device can be a client.

- **Server:** A Server is a Deeper Connect device that provides data proxy service to another Deeper Connect device. Only Deeper Connect device that has a public IP address can be a server.

- **Channel:** A micropayment channel between a client and a server. It's opened by a client and usually closed by a server, but a client can also close expired channels. A channel has a life span which is specified by the client in seconds when it opens the channel. A client also needs to lock a certain amount of DPR to open the channel. The amount of DPR is locked in the channel until the channel is closed. The amount of DRP in the channel is either claimed by the server or returned to the client when the channel is closed.

- **Nonce:** An index that indicates an occurring of an channel between the client and the server. It starts with 0 and increment by 1 each time. E.g., when Client A opens a channel to Server B for the first time, the nonce is 0. When the first channel is closed and Client A opens a channel to Server B again, the nonce becomes 1, and so on so forth. The Nonce of channel between Client C and Server B is independent and also starts with 0. Nonce is used to avoid duplicate channels between a client and a server at the same time.

- **SessionId:** Whenever a server claims payment from a channel, a session is ended. A server can claim payments from a channel multiple times, hence a channel can have multiple sessions. SessionId is unique in a channel and used to avoid duplicate charges.

## Interface

### Dispatchable Functions

- `open_channel` - a client opens a channel to a server.
- `close_channel` - a server closes a channel, or a client closes an expired channel.
- `close_expired_channels` - a client closes all its expired channels.
- `add_balance` - a client add more DPR to an existing channel.
- `claim_payment` - a server claims payment from a channel.

## Usage

This pallet only provides dispatchable functions to end users.

## Genesis config

There is nothing to config for this pallet in genesis.

## Dependencies

This pallet depends on the CreditInterface trait. It's not a general purpose pallet that can be used elsewhere, but we hope the concepts and interfaces can be useful for other projects.

License: Apache-2.0
