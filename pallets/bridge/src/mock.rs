use crate::{self as pallet_eth_sub_bridge, crypto::TestAuthId, ethereum};
use frame_support::{
    parameter_types,
    traits::{GenesisBuild, OnFinalize, OnInitialize},
};
use frame_system as system;
use frame_system::offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer};
use sp_core::H256;
use sp_core::{
    offchain::{
        testing::{self, OffchainState, PoolState},
        OffchainExt, TransactionPoolExt,
    },
    sr25519::{self, Signature},
    Pair, Public,
};
use sp_runtime::{
    testing::{Header, TestXt},
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};
use sp_runtime::{MultiSignature, MultiSigner};

use node_primitives::{Balance, BlockNumber, Moment};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type AccountPublic = <Signature as Verify>::Signer;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Event<T>, Config<T>},
        Bridge: pallet_eth_sub_bridge::{Module, Call, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = (); //pallet_balances::weights::SubstrateWeight<Test>;
}

pub const MILLISECS_PER_BLOCK: Moment = 5000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
parameter_types! {
    pub const MinimumPeriod: Moment = 2500u64;
    pub const BlocksPerEra: BlockNumber =  6 * EPOCH_DURATION_IN_BLOCKS;
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_eth_sub_bridge::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type BlocksPerEra = BlocksPerEra;
    type WeightInfo = ();
    type AuthorityId = TestAuthId;
    type Call = Call;
    type EthClient = ethereum::MockEthClient;
}

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn v1() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("V1")
}
pub fn v2() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("V2")
}
pub fn v3() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("V3")
}
pub fn v4() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("V4")
}

pub fn user1() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER1")
}
pub fn user2() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER2")
}
pub fn user3() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER3")
}
pub fn user4() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER4")
}
pub fn user5() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER5")
}
pub fn user6() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER6")
}
pub fn user7() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER7")
}
pub fn user8() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER8")
}
pub fn user9() -> AccountId {
    get_account_id_from_seed::<sr25519::Public>("USER9")
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let _ = pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (v1(), 100000),
            (v2(), 100000),
            (v3(), 100000),
            (user1(), 100000),
            (user2(), 300000),
        ],
    }
    .assimilate_storage(&mut storage);

    let _ = pallet_eth_sub_bridge::GenesisConfig::<Test> {
        validators_count: 3u32,
        validator_accounts: vec![v1(), v2(), v3()],
        current_limits: vec![100, 200, 50, 400, 10],
    }
    .assimilate_storage(&mut storage);

    let ext = sp_io::TestExternalities::from(storage);
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        System::on_finalize(System::block_number());
        Bridge::on_finalize(System::block_number());
        Timestamp::set_timestamp(6 * n);
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}
