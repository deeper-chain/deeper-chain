# Staking Module

The Staking module is used to manage funds at stake by network maintainers.

## Overview

The Staking module is the means by which a set of network maintainers (known as _authorities_ in
some contexts and _validators_ in others) are chosen based upon those who voluntarily place
funds under deposit. Under deposit, those funds are rewarded under normal operation but are held
at pain of _slash_ (expropriation) should the staked maintainer be found not to be discharging
its duties properly.

### Terminology
<!-- Original author of paragraph: @gavofyork -->

- Staking: The process of locking up funds for some time, placing them at risk of slashing
  (loss) in order to become a rewarded maintainer of the network.
- Validating: The process of running a node to actively maintain the network, either by
  producing blocks or guaranteeing finality of the chain.
- Stash account: The account holding an owner's funds used for Staking.
- Controller account: The account that controls an owner's funds for staking.
- Era: A (whole) number of sessions, which is the period that the validator set (and each
  validator's active nominator set) is recalculated and where rewards are paid out.
- Slash: The punishment of a staker by reducing its funds.

### Goals
<!-- Original author of paragraph: @gavofyork -->

The staking system in Substrate NPoS is designed to make the following possible:

- Stake funds that are controlled by a cold wallet.
- Withdraw some, or deposit more, funds without interrupting the role of an entity.
- Switch between roles (nominator, validator, idle) with minimal overhead.

### Scenarios

#### Staking

Almost any interaction with the Staking module requires a process of _**bonding**_ (also known
as being a _staker_). To become *bonded*, a fund-holding account known as the _stash account_,
which holds some or all of the funds that become frozen in place as part of the staking process,
is paired with an active **controller** account, which issues instructions on how they shall be
used.

An account pair can become bonded using the [`bond`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.bond) call.

Stash accounts can change their associated controller using the
[`set_controller`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.set_controller) call.

There are three possible roles that any staked account pair can be in: `Validator`, `Nominator`
and `Idle` (defined in [`StakerStatus`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.StakerStatus.html)). There are three
corresponding instructions to change between roles, namely:
[`validate`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.validate),
and [`chill`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.chill).

#### Validating

A **validator** takes the role of either validating blocks or ensuring their finality,
maintaining the veracity of the network. A validator should avoid both any sort of malicious
misbehavior and going offline. Bonded accounts that state interest in being a validator do NOT
get immediately chosen as a validator. Instead, they are declared as a _candidate_ and they
_might_ get elected at the _next era_ as a validator. The result of the election is determined
by nominators and their votes.

An account can become a validator candidate via the
[`validate`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.validate) call.

#### Delegation

A **delegator** does not take any _direct_ role in maintaining the network, instead, it votes on
a set of validators  to be elected. Once interest in nomination is stated by an account, it
takes effect at the next election round. The funds in the nominator's stash account indicate the
_weight_ of its vote. Both the rewards and any punishment that a validator earns are shared
between the validator and its delegators. This rule incentivizes the delegators to NOT vote for
the misbehaving/offline validators as much as possible, simply because the delegators will also
lose funds if they vote poorly.

#### Rewards and Slash

TODO
#### Chilling

Finally, any of the roles above can choose to step back temporarily and just chill for a while.
This means that if they are a delegator, they will not be considered as voters anymore and if
they are validators, they will no longer be a candidate for the next election.

An account can step back via the [`chill`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.chill) call.

### Session managing

The module implement the trait `SessionManager`. Which is the only API to query new validator
set and allowing these validator set to be rewarded once their era is ended.

## Interface

### Dispatchable Functions

The dispatchable functions of the Staking module enable the steps needed for entities to accept
and change their role, alongside some helper functions to get/set the metadata of the module.

### Public Functions

The Staking module contains many public storage items and (im)mutable functions.

## Implementation Details

### Era payout

TODO

### Reward Calculation

TODO
### Additional Fund Management Operations

Any funds already placed into stash can be the target of the following operations:

The controller account can free a portion (or all) of the funds using the
[`unbond`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.unbond) call. Note that the funds are not immediately
accessible. Instead, a duration denoted by [`BondingDuration`](https://docs.rs/pallet-staking/latest/pallet_staking/trait.Trait.html#associatedtype.BondingDuration)
(in number of eras) must pass until the funds can actually be removed. Once the
`BondingDuration` is over, the [`withdraw_unbonded`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.withdraw_unbonded)
call can be used to actually withdraw the funds.

Note that there is a limitation to the number of fund-chunks that can be scheduled to be
unlocked in the future via [`unbond`](https://docs.rs/pallet-staking/latest/pallet_staking/enum.Call.html#variant.unbond). In case this maximum
(`MAX_UNLOCKING_CHUNKS`) is reached, the bonded account _must_ first wait until a successful
call to `withdraw_unbonded` to remove some of the chunks.

### Election Algorithm

TODO

## GenesisConfig

The Staking module depends on the [`GenesisConfig`](https://docs.rs/pallet-staking/latest/pallet_staking/struct.GenesisConfig.html). The
`GenesisConfig` is optional and allow to set some initial stakers.

## Related Modules

- [Balances](https://docs.rs/pallet-balances/latest/pallet_balances/): Used to manage values at stake.
- [Session](https://docs.rs/pallet-session/latest/pallet_session/): Used to manage sessions. Also, a list of new
  validators is stored in the Session module's `Validators` at the end of each era.

License: Apache-2.0
