use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_std::prelude::Vec;

pub type MemberId = u64;
pub type ProposalId = u64;
pub type Days = u32;
pub type Rate = u32;

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Proposal<AccountId, Balance, VotingDeadline, MemberId> {
    pub action: Action<AccountId, Balance, VotingDeadline>,
    pub open: bool,
    pub accepted: bool,
    pub voting_deadline: VotingDeadline,
    pub yes_count: MemberId,
    pub no_count: MemberId,
}

impl<A, B, V, M> Default for Proposal<A, B, V, M>
where
    A: Default,
    B: Default,
    V: Default,
    M: Default,
{
    fn default() -> Self {
        Proposal {
            action: Action::EmptyAction,
            open: true,
            accepted: false,
            voting_deadline: V::default(),
            yes_count: M::default(),
            no_count: M::default(),
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Action<AccountId, Balance, Timeout> {
    EmptyAction,
    AddMember(AccountId),
    RemoveMember(AccountId),
    GetLoan(Vec<u8>, Days, Rate, Balance),
    ChangeTimeout(Timeout),
    ChangeMaximumNumberOfMembers(MemberId),
}

//bridge
#[derive(Encode, Decode, Clone, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Limits<Balance> {
    pub max_tx_value: Balance,
    pub day_max_limit: Balance,
    pub day_max_limit_for_one_address: Balance,
    pub max_pending_tx_limit: Balance,
    pub min_tx_value: Balance,
}

// bridge types
#[derive(Encode, Decode, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BridgeTransfer<Hash> {
    pub transfer_id: ProposalId,
    pub message_id: Hash,
    pub open: bool,
    pub votes: MemberId,
    pub kind: Kind,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, TypeInfo)]
pub enum Status {
    Revoked,
    Pending,
    PauseTheBridge,
    ResumeTheBridge,
    UpdateValidatorSet,
    UpdateLimits,
    Deposit,
    Withdraw,
    Approved,
    Canceled,
    Confirmed,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug, TypeInfo)]
pub enum Kind {
    Transfer,
    Limits,
    Validator,
    Bridge,
}

#[derive(Encode, Decode, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct TransferMessage<AccountId, Hash, Balance> {
    pub message_id: Hash,
    pub eth_address: H160,
    pub substrate_address: AccountId,
    pub amount: Balance,
    pub status: Status,
    pub action: Status,
}

#[derive(Encode, Decode, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct LimitMessage<Hash, Balance> {
    pub id: Hash,
    pub limits: Limits<Balance>,
    pub status: Status,
}

#[derive(Encode, Decode, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BridgeMessage<AccountId, Hash> {
    pub message_id: Hash,
    pub account: AccountId,
    pub action: Status,
    pub status: Status,
}

#[derive(Encode, Decode, Clone, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValidatorMessage<AccountId, Hash> {
    pub message_id: Hash,
    pub quorum: u64,
    pub accounts: Vec<AccountId>,
    pub action: Status,
    pub status: Status,
}

impl<A, H, B> Default for TransferMessage<A, H, B>
where
    A: Default,
    H: Default,
    B: Default,
{
    fn default() -> Self {
        TransferMessage {
            message_id: H::default(),
            eth_address: H160::default(),
            substrate_address: A::default(),
            amount: B::default(),
            status: Status::Withdraw,
            action: Status::Withdraw,
        }
    }
}

impl<H, B> Default for LimitMessage<H, B>
where
    H: Default,
    B: Default,
{
    fn default() -> Self {
        LimitMessage {
            id: H::default(),
            limits: Limits::default(),
            status: Status::UpdateLimits,
        }
    }
}

impl<A, H> Default for BridgeMessage<A, H>
where
    A: Default,
    H: Default,
{
    fn default() -> Self {
        BridgeMessage {
            message_id: H::default(),
            account: A::default(),
            action: Status::Revoked,
            status: Status::Revoked,
        }
    }
}

impl<A, H> Default for ValidatorMessage<A, H>
where
    A: Default,
    H: Default,
{
    fn default() -> Self {
        ValidatorMessage {
            message_id: H::default(),
            quorum: u64::default(),
            accounts: Vec::default(),
            action: Status::Revoked,
            status: Status::Revoked,
        }
    }
}

impl<H> Default for BridgeTransfer<H>
where
    H: Default,
{
    fn default() -> Self {
        BridgeTransfer {
            transfer_id: ProposalId::default(),
            message_id: H::default(),
            open: true,
            votes: MemberId::default(),
            kind: Kind::Transfer,
        }
    }
}

impl<B> Default for Limits<B>
where
    B: Default,
{
    fn default() -> Self {
        Limits {
            max_tx_value: B::default(),
            day_max_limit: B::default(),
            day_max_limit_for_one_address: B::default(),
            max_pending_tx_limit: B::default(),
            min_tx_value: B::default(),
        }
    }
}

pub trait IntoArray<T> {
    fn into_array(&self) -> [T; 5];
}

impl<B: Clone> IntoArray<B> for Limits<B> {
    fn into_array(&self) -> [B; 5] {
        [
            self.max_tx_value.clone(),
            self.day_max_limit.clone(),
            self.day_max_limit_for_one_address.clone(),
            self.max_pending_tx_limit.clone(),
            self.min_tx_value.clone(),
        ]
    }
}
