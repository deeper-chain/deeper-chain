#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod ethereum;
mod types;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
use sp_std::prelude::*;
pub mod weights;
use ethereum::Client;
use weights::WeightInfo;

pub mod crypto {
    use sp_core::crypto::KeyTypeId;

    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    const KEY_TYPE: KeyTypeId = KeyTypeId(*b"brgt"); // bridge test

    app_crypto!(sr25519, KEY_TYPE);

    pub struct TestAuthId;

    // implemented for runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for TestAuthId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

pub mod sr25519 {
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{traits::Verify, MultiSignature, MultiSigner};
    mod app_sr25519 {
        use sp_application_crypto::{app_crypto, key_types::BABE, sr25519};
        app_crypto!(sr25519, BABE);
    }

    pub type AuthorityId = app_sr25519::Public;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthorityId {
        type RuntimeAppPublic = app_sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for AuthorityId
    {
        type RuntimeAppPublic = app_sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use codec::{Decode, Encode};
    use frame_support::traits::{Currency, ReservableCurrency};
    use frame_support::{dispatch::DispatchResultWithPostInfo, fail, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use serde::{Deserialize, Serialize};
    use sp_core::H160;
    use sp_runtime::traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Hash, Zero};
    use sp_std::prelude::Vec;
    use sp_std::str;
    use types::{
        BridgeMessage, BridgeTransfer, IntoArray, Kind, LimitMessage, Limits, MemberId, ProposalId,
        Status, TransferMessage, ValidatorMessage,
    };

    use frame_system::offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer,
    };
    use sp_runtime::offchain::{
        http,
        storage::StorageValueRef,
        storage_lock::{BlockAndTime, BlockNumberProvider, StorageLock},
        Duration,
    };

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    const MAX_VALIDATORS: u32 = 100_000;
    const DAY: u32 = 86_400;

    const HTTP_REMOTE_REQUEST: &str =
        "https://mainnet.infura.io/v3/75284d8d0fb14ab88520b949270fe205";
    const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds
    const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD + 1000; // in milli-seconds
    const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_timestamp::Config + CreateSignedTransaction<Call<Self>>
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        type BlocksPerEra: Get<<Self as frame_system::Config>::BlockNumber>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
        type Call: From<Call<Self>>;
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::type_value]
    pub fn DefaultBridgeIsOperational<T: Config>() -> bool {
        true
    }
    #[pallet::storage]
    #[pallet::getter(fn bridge_is_operational)]
    pub type BridgeIsOperational<T> =
        StorageValue<_, bool, ValueQuery, DefaultBridgeIsOperational<T>>;

    #[pallet::storage]
    #[pallet::getter(fn bridge_messages)]
    pub type BridgeMessages<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BridgeMessage<T::AccountId, T::Hash>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn limit_messages)]
    pub type LimitMessages<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, LimitMessage<T::Hash, BalanceOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_limits)]
    pub type CurrentLimits<T: Config> = StorageValue<_, Limits<BalanceOf<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_burn_count)]
    pub type CurrentPendingBurn<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_mint_count)]
    pub type CurrentPendingMint<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn transfers)]
    pub type BridgeTransfers<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalId, BridgeTransfer<T::Hash>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bridge_transfers_count)]
    pub type BridgeTransfersCount<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn messages)]
    pub type TransferMessages<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn transfer_id_by_hash)]
    pub type TransferId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, ProposalId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn message_id_by_transfer_id)]
    pub type MessageId<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalId, T::Hash, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn daily_holds)]
    pub type DailyHolds<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (T::BlockNumber, T::Hash), ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn daily_limits_by_account)]
    pub type DailyLimits<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn daily_blocked)]
    pub type DailyBlocked<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Moment, Vec<T::AccountId>, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultQuorum<T: Config>() -> u64 {
        2u64
    }
    #[pallet::storage]
    #[pallet::getter(fn quorum)]
    pub type Quorum<T> = StorageValue<_, u64, ValueQuery, DefaultQuorum<T>>;

    #[pallet::storage]
    #[pallet::getter(fn validators_count)]
    pub type ValidatorsCount<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validator_votes)]
    pub type ValidatorVotes<T: Config> =
        StorageMap<_, Blake2_128Concat, (ProposalId, T::AccountId), bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validator_history)]
    pub type ValidatorHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        ValidatorMessage<T::AccountId, T::Hash>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validator_accounts)]
    pub type ValidatorAccounts<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    /// key is infura request url, value is filter_id
    #[pallet::storage]
    #[pallet::getter(fn eth_filter_ids)]
    pub type EthFilterIds<T> = StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<u8>, ValueQuery>;

    // #[derive(
    //     Serialize,
    //     Deserialize,
    //     Decode,
    //     Encode,
    //     Default,
    //     Clone,
    //     Debug,
    //     PartialEq,
    //     Eq,
    //     Ord,
    //     PartialOrd,
    // )]
    // pub struct EthDprTransaction<T: Config> {
    //     pub message_id: T::Hash,
    //     pub sender: H160,
    //     pub reciver: T::AccountId,
    //     pub amount: BalanceOf<T>,
    // }

    #[pallet::storage]
    #[pallet::getter(fn eth_dpr_transactions)]
    pub type EthDprTransactions<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, ethereum::SetTransferData, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub current_limits: Vec<BalanceOf<T>>,
        pub validators_count: u32,
        pub validator_accounts: Vec<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                current_limits: Default::default(),
                validators_count: 3u32,
                validator_accounts: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            let mut limits_iter = self.current_limits.clone().into_iter();
            let limits = Limits {
                max_tx_value: limits_iter.next().unwrap(),
                day_max_limit: limits_iter.next().unwrap(),
                day_max_limit_for_one_address: limits_iter.next().unwrap(),
                max_pending_tx_limit: limits_iter.next().unwrap(),
                min_tx_value: limits_iter.next().unwrap(),
            };
            <CurrentLimits<T>>::put(limits);

            <ValidatorsCount<T>>::put(self.validators_count);

            <ValidatorAccounts<T>>::put(&self.validator_accounts);
            for v in &self.validator_accounts {
                <Validators<T>>::insert(v, true);
            }
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RelayMessage(T::Hash),
        ApprovedRelayMessage(T::Hash, T::AccountId, H160, BalanceOf<T>),
        CancellationConfirmedMessage(T::Hash),
        MintedMessage(T::Hash),
        BurnedMessage(T::Hash, T::AccountId, H160, BalanceOf<T>),
        AccountPausedMessage(T::Hash, T::AccountId, T::Moment),
        AccountResumedMessage(T::Hash, T::AccountId, T::Moment),
        ApprovedEthDprTransaction(T::Hash, T::AccountId, H160, BalanceOf<T>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        HttpFetchingError,
        GetLockError,
        DeserializeError,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            // sync eth->dpr transactions every 100 blocks
            if block_number % 2u32.into() == Zero::zero() {
                // let (logs, filter_id) = Self::get_eth_logs().unwrap();
                let client = ethereum::DefaultEthClient::default();
                let (logs, filter_id) = client.get_eth_logs().unwrap();
                let signer = Signer::<T, T::AuthorityId>::any_account();
                if !signer.can_sign() {
                    log::error!(
                        "No local accounts available. Consider adding one via `author_insertKey` RPC"
                    );
                    return;
                }
                let results = signer.send_signed_transaction(|_account| {
                    Call::submit_logs(logs.clone(), filter_id.clone())
                });
                for (acc, res) in &results {
                    match res {
                        Ok(()) => log::info!("submit_logs [{:?}] after_callback_ok", acc.id),
                        Err(e) => {
                            log::error!("submit_logs [{:?}] after_callback_error: {:?}", acc.id, e)
                        }
                    }
                }
            }
        }

        fn on_finalize(_n: T::BlockNumber) {
            // clear accounts blocked day earlier (e.g. 18759 - 1)
            let yesterday = Self::get_day_pair().0;
            let is_first_day = Self::get_day_pair().1 == yesterday;

            if <DailyBlocked<T>>::contains_key(yesterday) && !is_first_day {
                let blocked_yesterday = <DailyBlocked<T>>::get(yesterday);
                blocked_yesterday
                    .iter()
                    .for_each(|a| <DailyLimits<T>>::remove(a));
                blocked_yesterday.iter().for_each(|a| {
                    let now = <pallet_timestamp::Module<T>>::get();
                    let hash = (now.clone(), a.clone())
                        .using_encoded(<T as frame_system::Config>::Hashing::hash);
                    Self::deposit_event(Event::AccountResumedMessage(hash, a.clone(), now));
                });
                <DailyBlocked<T>>::remove(yesterday);
            }
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResultWithPostInfo.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(<T as pallet::Config>::WeightInfo::submit_logs())]
        pub fn submit_logs(
            origin: OriginFor<T>,
            logs: Vec<ethereum::SetTransferData>,
            filter_id: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // Retrieve sender of the transaction.
            let who = ensure_signed(origin)?;
            for log in logs.iter() {
                if log.message_id.is_empty() {
                    continue;
                }
                // let recipient = ensure_signed(log.recipient)?;

                let message_id = log
                    .message_id
                    .using_encoded(<T as frame_system::Config>::Hashing::hash);

                if !<EthDprTransactions<T>>::contains_key(message_id) {
                    <EthDprTransactions<T>>::insert(message_id, log);

                    let amount = log.amount as u32;
                    let amount = BalanceOf::<T>::from(amount);
                    let sender = H160::from_slice(&log.sender);
                    // T::Currency::deposit_creating(&recipient, amount); // mint

                    Self::deposit_event(Event::ApprovedEthDprTransaction(
                        message_id,
                        who.clone(),
                        sender,
                        amount,
                    ));
                }
            }
            log::info!(
                "eth_filter_ids: {:?}, {:?}",
                HTTP_REMOTE_REQUEST.as_bytes().to_vec(),
                filter_id.clone()
            );

            // save filter_id back online
            <EthFilterIds<T>>::mutate(HTTP_REMOTE_REQUEST.as_bytes().to_vec(), |v| *v = filter_id);

            Ok(().into())
        }

        // initiate substrate -> ethereum transfer.
        // create transfer and emit the RelayMessage event
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_transfer())]
        pub fn set_transfer(
            origin: OriginFor<T>,
            to: H160,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");

            Self::check_amount(amount)?;
            Self::check_pending_burn(amount)?;
            Self::check_daily_account_volume(from.clone(), amount)?;

            let transfer_hash = (&from, &to, amount, <pallet_timestamp::Module<T>>::get())
                .using_encoded(<T as frame_system::Config>::Hashing::hash);

            let message = TransferMessage {
                message_id: transfer_hash,
                eth_address: to,
                substrate_address: from.clone(),
                amount,
                status: Status::Withdraw,
                action: Status::Withdraw,
            };
            Self::get_transfer_id_checked(transfer_hash, Kind::Transfer)?;
            Self::deposit_event(Event::RelayMessage(transfer_hash));

            <DailyLimits<T>>::mutate(from, |a| *a += amount);
            <TransferMessages<T>>::insert(transfer_hash, message);
            Ok(().into())
        }

        // ethereum-side multi-signed mint operation
        #[pallet::weight(<T as pallet::Config>::WeightInfo::multi_signed_mint())]
        pub fn multi_signed_mint(
            origin: OriginFor<T>,
            message_id: T::Hash,
            from: H160,
            to: T::AccountId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");

            Self::check_validator(validator.clone())?;
            Self::check_pending_mint(amount)?;
            Self::check_amount(amount)?;

            if !<TransferMessages<T>>::contains_key(message_id) {
                let message = TransferMessage {
                    message_id,
                    eth_address: from,
                    substrate_address: to,
                    amount,
                    status: Status::Deposit,
                    action: Status::Deposit,
                };
                <TransferMessages<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Transfer)?;
            }

            let transfer_id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, transfer_id)?;
            Ok(().into())
        }

        // change maximum tx limit
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_limits())]
        pub fn update_limits(
            origin: OriginFor<T>,
            max_tx_value: BalanceOf<T>,
            day_max_limit: BalanceOf<T>,
            day_max_limit_for_one_address: BalanceOf<T>,
            max_pending_tx_limit: BalanceOf<T>,
            min_tx_value: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;
            let limits = Limits {
                max_tx_value,
                day_max_limit,
                day_max_limit_for_one_address,
                max_pending_tx_limit,
                min_tx_value,
            };
            Self::check_limits(&limits)?;
            let id = (limits.clone(), T::BlockNumber::from(0u32))
                .using_encoded(<T as frame_system::Config>::Hashing::hash);

            if !<LimitMessages<T>>::contains_key(id) {
                let message = LimitMessage {
                    id,
                    limits,
                    status: Status::UpdateLimits,
                };
                <LimitMessages<T>>::insert(id, message);
                Self::get_transfer_id_checked(id, Kind::Limits)?;
            }

            let transfer_id = <TransferId<T>>::get(id);
            Self::_sign(validator, transfer_id)?;
            Ok(().into())
        }

        // validator`s response to RelayMessage
        #[pallet::weight(<T as pallet::Config>::WeightInfo::approve_transfer())]
        pub fn approve_transfer(
            origin: OriginFor<T>,
            message_id: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;

            let id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, id)?;
            Ok(().into())
        }

        // each validator calls it to update whole set of validators
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_validator_list())]
        pub fn update_validator_list(
            origin: OriginFor<T>,
            message_id: T::Hash,
            quorum: u64,
            new_validator_list: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            if !<ValidatorHistory<T>>::contains_key(message_id) {
                let message = ValidatorMessage {
                    message_id,
                    quorum,
                    accounts: new_validator_list,
                    action: Status::UpdateValidatorSet,
                    status: Status::UpdateValidatorSet,
                };
                <ValidatorHistory<T>>::insert(message_id, message);
                Self::get_transfer_id_checked(message_id, Kind::Validator)?;
            }

            let id = <TransferId<T>>::get(message_id);
            Self::_sign(validator, id)?;
            Ok(().into())
        }

        // each validator calls it to pause the bridge
        #[pallet::weight(<T as pallet::Config>::WeightInfo::pause_bridge())]
        pub fn pause_bridge(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            ensure!(
                Self::bridge_is_operational(),
                "Bridge is not operational already"
            );
            let hash = ("pause", T::BlockNumber::from(0u32))
                .using_encoded(<T as frame_system::Config>::Hashing::hash);

            if !<BridgeMessages<T>>::contains_key(hash) {
                let message = BridgeMessage {
                    message_id: hash,
                    account: validator.clone(),
                    action: Status::PauseTheBridge,
                    status: Status::PauseTheBridge,
                };
                <BridgeMessages<T>>::insert(hash, message);
                Self::get_transfer_id_checked(hash, Kind::Bridge)?;
            }

            let id = <TransferId<T>>::get(hash);
            Self::_sign(validator, id)?;
            Ok(().into())
        }

        // each validator calls it to resume the bridge
        #[pallet::weight(<T as pallet::Config>::WeightInfo::resume_bridge())]
        pub fn resume_bridge(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            let hash = ("resume", T::BlockNumber::from(0u32))
                .using_encoded(<T as frame_system::Config>::Hashing::hash);

            if !<BridgeMessages<T>>::contains_key(hash) {
                let message = BridgeMessage {
                    message_id: hash,
                    account: validator.clone(),
                    action: Status::ResumeTheBridge,
                    status: Status::ResumeTheBridge,
                };
                <BridgeMessages<T>>::insert(hash, message);
                Self::get_transfer_id_checked(hash, Kind::Bridge)?;
            }

            let id = <TransferId<T>>::get(hash);
            Self::_sign(validator, id)?;
            Ok(().into())
        }

        //confirm burn from validator
        #[pallet::weight(<T as pallet::Config>::WeightInfo::confirm_transfer())]
        pub fn confirm_transfer(
            origin: OriginFor<T>,
            message_id: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            ensure!(Self::bridge_is_operational(), "Bridge is not operational");
            Self::check_validator(validator.clone())?;

            let id = <TransferId<T>>::get(message_id);

            let is_approved = <TransferMessages<T>>::get(message_id).status == Status::Approved
                || <TransferMessages<T>>::get(message_id).status == Status::Confirmed;
            ensure!(is_approved, "This transfer must be approved first.");

            Self::update_status(message_id, Status::Confirmed, Kind::Transfer)?;
            Self::reopen_for_burn_confirmation(message_id)?;
            Self::_sign(validator, id)?;
            Ok(().into())
        }

        //cancel burn from validator
        #[pallet::weight(<T as pallet::Config>::WeightInfo::cancel_transfer())]
        pub fn cancel_transfer(
            origin: OriginFor<T>,
            message_id: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let validator = ensure_signed(origin)?;
            Self::check_validator(validator.clone())?;

            let has_burned = <TransferMessages<T>>::contains_key(message_id)
                && <TransferMessages<T>>::get(message_id).status == Status::Confirmed;
            ensure!(
                !has_burned,
                "Failed to cancel. This transfer is already executed."
            );

            let id = <TransferId<T>>::get(message_id);
            Self::update_status(message_id, Status::Canceled, Kind::Transfer)?;
            Self::reopen_for_burn_confirmation(message_id)?;
            Self::_sign(validator, id)?;
            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn get_eth_logs() -> Result<(Vec<ethereum::SetTransferData>, Vec<u8>), Error<T>> {
            // Create a reference to Local Storage value.
            // Since the local storage is common for all offchain workers, it's a good practice
            // to prepend our entry with the pallet name.
            let s_info = StorageValueRef::persistent(b"pallet-eth-sub-bridge::get-eth-logs");

            // Local storage is persisted and shared between runs of the offchain workers,
            // offchain workers may run concurrently. We can use the `mutate` function to
            // write a storage entry in an atomic fashion.
            //
            // With a similar API as `StorageValue` with the variables `get`, `set`, `mutate`.
            // We will likely want to use `mutate` to access
            // the storage comprehensively.
            //
            if let Some(Some(info)) = s_info.get::<(Vec<ethereum::SetTransferData>, Vec<u8>)>() {
                // hn-info has already been fetched. Return early.
                log::info!("cached hn-info: {:?}", info);
                return Ok(info);
            }

            // Since off-chain storage can be accessed by off-chain workers from multiple runs, it is important to lock
            //   it before doing heavy computations or write operations.
            //
            // There are four ways of defining a lock:
            //   1) `new` - lock with default time and block exipration
            //   2) `with_deadline` - lock with default block but custom time expiration
            //   3) `with_block_deadline` - lock with default time but custom block expiration
            //   4) `with_block_and_time_deadline` - lock with custom time and block expiration
            // Here we choose the most custom one for demonstration purpose.
            let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
                b"pallet-eth-sub-bridge::lock::get-eth-logs",
                LOCK_BLOCK_EXPIRATION,
                Duration::from_millis(LOCK_TIMEOUT_EXPIRATION),
            );

            // We try to acquire the lock here. If failed, we know the `fetch_n_parse` part inside is being
            //   executed by previous run of ocw, so the function just returns.
            if let Ok(_guard) = lock.try_lock() {
                match Self::get_eth_logs_n_parse() {
                    Ok((info, filter_id)) => {
                        let data: Vec<ethereum::SetTransferData> = info
                            .result
                            .iter()
                            .map(|d| ethereum::decode_data(&d.data))
                            .collect();
                        s_info.set(&data);
                        return Ok((data, filter_id));
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
            Err(<Error<T>>::GetLockError)
        }

        fn get_eth_logs_n_parse() -> Result<(ethereum::GetLogsResp, Vec<u8>), Error<T>> {
            let (resp_bytes, filter_id) = Self::get_eth_logs_from_remote().map_err(|e| {
                log::error!("fetch_from_remote error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;

            let resp_str =
                str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
            log::info!("get_eth_logs_n_parse: {}", resp_str);

            // Deserializing JSON to struct, thanks to `serde` and `serde_derive`
            let info: ethereum::GetLogsResp =
                serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError)?;
            Ok((info, filter_id))
        }

        /// This function uses the `offchain::http` API to query the remote endpoint information,
        ///   and returns the JSON response as vector of bytes.
        fn get_eth_logs_from_remote() -> Result<(Vec<u8>, Vec<u8>), Error<T>> {
            let mut filter_id = <EthFilterIds<T>>::get(HTTP_REMOTE_REQUEST.as_bytes().to_vec());
            if !<EthFilterIds<T>>::contains_key(&HTTP_REMOTE_REQUEST.as_bytes().to_vec()) {
                if let Ok(id) = Self::create_eth_filter_id() {
                    filter_id = id;
                }
            }

            // string format doesn't work in no_std, so concat Vec<u8> to construct the http request body
            let mut body_bytes =
                "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getFilterChanges\",\"params\":[\""
                    .as_bytes()
                    .to_vec();
            body_bytes.append(&mut filter_id);
            body_bytes.append(&mut "\"],\"id\":1}".as_bytes().to_vec());
            let body_str = str::from_utf8(&body_bytes).unwrap();
            let body = vec![body_str];
            log::info!("get_eth_logs_from_remote: {:?}", body);

            let request = http::Request::post(HTTP_REMOTE_REQUEST, body);

            // Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
            let timeout =
                sp_io::offchain::timestamp().add(Duration::from_millis(FETCH_TIMEOUT_PERIOD));

            let pending = request
                .add_header("Content-Type", "application/json")
                .deadline(timeout) // Setting the timeout time
                .send() // Sending the request out by the host
                .map_err(|e| {
                    log::error!("{:?}", e);
                    <Error<T>>::HttpFetchingError
                })?;

            // By default, the http request is async from the runtime perspective. So we are asking the
            //   runtime to wait here
            // The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
            //   ref: https://docs.substrate.io/rustdocs/latest/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
            let response = pending
                .try_wait(timeout)
                .map_err(|e| {
                    log::error!("{:?}", e);
                    <Error<T>>::HttpFetchingError
                })?
                .map_err(|e| {
                    log::error!("{:?}", e);
                    <Error<T>>::HttpFetchingError
                })?;

            if response.code != 200 {
                log::error!("Unexpected http request status code: {}", response.code);
                return Err(<Error<T>>::HttpFetchingError);
            }

            // Next we fully read the response body and collect it to a vector of bytes.
            Ok((response.body().collect::<Vec<u8>>(), filter_id))
        }

        fn create_eth_filter_id() -> Result<Vec<u8>, Error<T>> {
            let s_info =
                StorageValueRef::persistent(b"pallet-eth-sub-bridge::create-eth-filter-id");

            // Local storage is persisted and shared between runs of the offchain workers,
            // offchain workers may run concurrently. We can use the `mutate` function to
            // write a storage entry in an atomic fashion.
            //
            // With a similar API as `StorageValue` with the variables `get`, `set`, `mutate`.
            // We will likely want to use `mutate` to access
            // the storage comprehensively.
            //
            if let Some(Some(info)) = s_info.get::<Vec<u8>>() {
                log::info!("cached new-filter-info: {:?}", info);
                return Ok(info);
            }

            let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
                b"pallet-eth-sub-bridge::lock::create-eth-filter-id",
                LOCK_BLOCK_EXPIRATION,
                Duration::from_millis(LOCK_TIMEOUT_EXPIRATION),
            );

            if let Ok(_guard) = lock.try_lock() {
                match Self::create_eth_filter_id_n_parse() {
                    Ok(info) => {
                        s_info.set(&info);
                        return Ok(info);
                    }
                    Err(err) => {
                        log::info!("create_eth_filter_id error, {:?}", err);
                        return Err(err);
                    }
                }
            }
            Err(<Error<T>>::GetLockError)
        }

        /// Fetch from remote and deserialize the JSON to a struct
        fn create_eth_filter_id_n_parse() -> Result<Vec<u8>, Error<T>> {
            let resp_bytes = Self::create_eth_filter_id_from_remote().map_err(|e| {
                log::error!("create_new_filter_id_n_parse error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;

            let resp_str =
                str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
            log::info!("create_new_filter_id_n_parse: {}", resp_str);

            let filter_id = parse_new_eth_filter_response(resp_str);
            if !filter_id.is_empty() {
                return Ok(filter_id);
            }
            Err(<Error<T>>::DeserializeError)
        }

        fn create_eth_filter_id_from_remote() -> Result<Vec<u8>, Error<T>> {
            // the topic_id is hard coded, see:https://etherscan.io/tx/0x1f3387c160289cf864d7f5e0ba8b87793095f404c0a19ec00aa1f1c7f581b7dc#eventlog
            let body = vec![
                r#"{"jsonrpc":"2.0","method":"eth_newFilter","params":[{"topics": ["0xfb65d1544ea97e32c62baf55f738f7bb44671998c927415ef03e52d2477e292f"]}],"id":1}"#,
            ];
            let request = http::Request::post(HTTP_REMOTE_REQUEST, body);

            // Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
            let timeout =
                sp_io::offchain::timestamp().add(Duration::from_millis(FETCH_TIMEOUT_PERIOD));

            let pending = request
                .add_header("Content-Type", "application/json")
                .deadline(timeout) // Setting the timeout time
                .send() // Sending the request out by the host
                .map_err(|e| {
                    log::error!("{:?}", e);
                    <Error<T>>::HttpFetchingError
                })?;

            // By default, the http request is async from the runtime perspective. So we are asking the
            //   runtime to wait here
            // The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
            //   ref: https://docs.substrate.io/rustdocs/latest/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
            let response = pending
                .try_wait(timeout)
                .map_err(|e| {
                    log::error!("{:?}", e);
                    <Error<T>>::HttpFetchingError
                })?
                .map_err(|e| {
                    log::error!("{:?}", e);
                    <Error<T>>::HttpFetchingError
                })?;

            if response.code != 200 {
                log::error!("Unexpected http request status code: {}", response.code);
                return Err(<Error<T>>::HttpFetchingError);
            }

            // Next we fully read the response body and collect it to a vector of bytes.
            Ok(response.body().collect::<Vec<u8>>())
        }

        fn _sign(validator: T::AccountId, transfer_id: ProposalId) -> Result<(), &'static str> {
            let mut transfer = <BridgeTransfers<T>>::get(transfer_id);

            let mut message = <TransferMessages<T>>::get(transfer.message_id);
            let mut limit_message = <LimitMessages<T>>::get(transfer.message_id);
            let mut validator_message = <ValidatorHistory<T>>::get(transfer.message_id);
            let mut bridge_message = <BridgeMessages<T>>::get(transfer.message_id);
            let voted = <ValidatorVotes<T>>::get((transfer_id, validator.clone()));
            ensure!(!voted, "This validator has already voted.");
            ensure!(transfer.open, "This transfer is not open");
            transfer.votes += 1;

            if Self::votes_are_enough(transfer.votes) {
                match message.status {
                    Status::Confirmed | Status::Canceled => (), // if burn is confirmed or canceled
                    _ => match transfer.kind {
                        Kind::Transfer => message.status = Status::Approved,
                        Kind::Limits => limit_message.status = Status::Approved,
                        Kind::Validator => validator_message.status = Status::Approved,
                        Kind::Bridge => bridge_message.status = Status::Approved,
                    },
                }
                match transfer.kind {
                    Kind::Transfer => Self::execute_transfer(message)?,
                    Kind::Limits => Self::_update_limits(limit_message)?,
                    Kind::Validator => Self::manage_validator_list(validator_message)?,
                    Kind::Bridge => Self::manage_bridge(bridge_message)?,
                }
                transfer.open = false;
            } else {
                match message.status {
                    Status::Confirmed | Status::Canceled => (),
                    _ => Self::set_pending(transfer_id, transfer.kind.clone())?,
                };
            }

            <ValidatorVotes<T>>::mutate((transfer_id, validator), |a| *a = true);
            <BridgeTransfers<T>>::insert(transfer_id, transfer);

            Ok(())
        }

        //     ///get (yesterday,today) pair
        fn get_day_pair() -> (T::Moment, T::Moment) {
            let now = <pallet_timestamp::Module<T>>::get();
            let day = T::Moment::from(DAY);
            let today = <pallet_timestamp::Module<T>>::get() / T::Moment::from(DAY);
            let yesterday = if now < day {
                T::Moment::from(0u32)
            } else {
                <pallet_timestamp::Module<T>>::get() / day - T::Moment::from(1u32)
            };
            (yesterday, today)
        }

        ///ensure that such transfer exist
        fn get_transfer_id_checked(transfer_hash: T::Hash, kind: Kind) -> Result<(), &'static str> {
            if !<TransferId<T>>::contains_key(transfer_hash) {
                Self::create_transfer(transfer_hash, kind)?;
            }
            Ok(())
        }

        //     ///execute actual mint
        fn deposit(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            Self::sub_pending_mint(message.clone())?;
            let to = message.substrate_address;
            if !<DailyHolds<T>>::contains_key(&to) {
                <DailyHolds<T>>::insert(
                    to.clone(),
                    (T::BlockNumber::from(0u32), message.message_id),
                );
            }

            T::Currency::deposit_creating(&to, message.amount); // mint

            Self::deposit_event(Event::MintedMessage(message.message_id));
            Self::update_status(message.message_id, Status::Confirmed, Kind::Transfer)
        }

        fn withdraw(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            Self::check_daily_holds(message.clone())?;
            Self::sub_pending_burn(message.clone())?;

            let to = message.eth_address;
            let from = message.substrate_address.clone();
            Self::lock_for_burn(&message, from.clone())?;
            Self::deposit_event(Event::ApprovedRelayMessage(
                message.message_id,
                from,
                to,
                message.amount,
            ));
            Self::update_status(message.message_id, Status::Approved, Kind::Transfer)
        }

        fn _cancel_transfer(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            T::Currency::unreserve(&message.substrate_address, message.amount); // unlock
            Self::update_status(message.message_id, Status::Canceled, Kind::Transfer)
        }

        fn pause_the_bridge(
            message: BridgeMessage<T::AccountId, T::Hash>,
        ) -> Result<(), &'static str> {
            <BridgeIsOperational<T>>::mutate(|x| *x = false);
            Self::update_status(message.message_id, Status::Confirmed, Kind::Bridge)
        }

        fn resume_the_bridge(
            message: BridgeMessage<T::AccountId, T::Hash>,
        ) -> Result<(), &'static str> {
            <BridgeIsOperational<T>>::mutate(|x| *x = true);
            Self::update_status(message.message_id, Status::Confirmed, Kind::Bridge)
        }

        fn _update_limits(
            message: LimitMessage<T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            Self::check_limits(&message.limits)?;
            <CurrentLimits<T>>::put(message.limits);
            Self::update_status(message.id, Status::Confirmed, Kind::Limits)
        }

        fn add_pending_burn(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            let current = <CurrentPendingBurn<T>>::get();
            let next = current
                .checked_add(&message.amount)
                .ok_or("Overflow adding to new pending burn volume")?;
            <CurrentPendingBurn<T>>::put(next);
            Ok(())
        }

        fn add_pending_mint(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            let current = <CurrentPendingMint<T>>::get();
            let next = current
                .checked_add(&message.amount)
                .ok_or("Overflow adding to new pending mint volume")?;
            <CurrentPendingMint<T>>::put(next);
            Ok(())
        }

        fn sub_pending_burn(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            let current = <CurrentPendingBurn<T>>::get();
            let next = current
                .checked_sub(&message.amount)
                .ok_or("Overflow subtracting to new pending burn volume")?;
            <CurrentPendingBurn<T>>::put(next);
            Ok(())
        }

        fn sub_pending_mint(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            let current = <CurrentPendingMint<T>>::get();
            let next = current
                .checked_sub(&message.amount)
                .ok_or("Overflow subtracting to new pending mint volume")?;
            <CurrentPendingMint<T>>::put(next);
            Ok(())
        }

        //     /// update validators list
        fn manage_validator_list(
            info: ValidatorMessage<T::AccountId, T::Hash>,
        ) -> Result<(), &'static str> {
            let new_count = info.accounts.clone().len() as u32;
            ensure!(
                new_count < MAX_VALIDATORS,
                "New validator list is exceeding allowed length."
            );
            <Quorum<T>>::put(info.quorum);
            <ValidatorsCount<T>>::put(new_count);
            info.accounts
                .clone()
                .iter()
                .for_each(|v| <Validators<T>>::insert(v, true));
            Self::update_status(info.message_id, Status::Confirmed, Kind::Validator)
        }

        /// check votes validity
        fn votes_are_enough(votes: MemberId) -> bool {
            votes as f64 / f64::from(Self::validators_count()) >= 0.51
        }

        /// lock funds after set_transfer call
        fn lock_for_burn(
            message: &TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
            account: T::AccountId,
        ) -> Result<(), &'static str> {
            let _ = T::Currency::reserve(&account, message.amount)?;
            Ok(())
        }

        fn execute_burn(message_id: T::Hash) -> Result<(), &'static str> {
            let message = <TransferMessages<T>>::get(message_id);
            let from = message.substrate_address.clone();
            let to = message.eth_address;
            let (_, res_bal) = T::Currency::slash_reserved(&from, message.amount); // burn
            ensure!(res_bal == (BalanceOf::<T>::zero()), "slash_reserved failed");
            <DailyLimits<T>>::mutate(from.clone(), |a| *a -= message.amount);

            Self::deposit_event(Event::BurnedMessage(message_id, from, to, message.amount));
            Ok(())
        }

        fn execute_transfer(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            match message.action {
                Status::Deposit => match message.status {
                    Status::Approved => Self::deposit(message),
                    Status::Canceled => Self::_cancel_transfer(message),
                    _ => Err("Tried to deposit with non-supported status"),
                },
                Status::Withdraw => match message.status {
                    Status::Confirmed => Self::execute_burn(message.message_id),
                    Status::Approved => Self::withdraw(message),
                    Status::Canceled => Self::_cancel_transfer(message),
                    _ => Err("Tried to withdraw with non-supported status"),
                },
                _ => Err("Tried to execute transfer with non-supported status"),
            }
        }

        fn manage_bridge(
            message: BridgeMessage<T::AccountId, T::Hash>,
        ) -> Result<(), &'static str> {
            match message.action {
                Status::PauseTheBridge => match message.status {
                    Status::Approved => Self::pause_the_bridge(message),
                    _ => Err("Tried to pause the bridge with non-supported status"),
                },
                Status::ResumeTheBridge => match message.status {
                    Status::Approved => Self::resume_the_bridge(message),
                    _ => Err("Tried to resume the bridge with non-supported status"),
                },
                _ => Err("Tried to manage bridge with non-supported status"),
            }
        }

        fn create_transfer(transfer_hash: T::Hash, kind: Kind) -> Result<(), &'static str> {
            ensure!(
                !<TransferId<T>>::contains_key(transfer_hash),
                "This transfer already open"
            );

            let transfer_id = <BridgeTransfersCount<T>>::get();
            let bridge_transfers_count = <BridgeTransfersCount<T>>::get();
            let new_bridge_transfers_count = bridge_transfers_count
                .checked_add(1)
                .ok_or("Overflow adding a new bridge transfer")?;
            let transfer = crate::types::BridgeTransfer {
                transfer_id,
                message_id: transfer_hash,
                open: true,
                votes: 0,
                kind,
            };

            <BridgeTransfers<T>>::insert(transfer_id, transfer);
            <BridgeTransfersCount<T>>::mutate(|count| *count = new_bridge_transfers_count);
            <TransferId<T>>::insert(transfer_hash, transfer_id);
            <MessageId<T>>::insert(transfer_id, transfer_hash);

            Ok(())
        }

        fn set_pending(transfer_id: ProposalId, kind: Kind) -> Result<(), &'static str> {
            let message_id = <MessageId<T>>::get(transfer_id);
            match kind {
                Kind::Transfer => {
                    let message = <TransferMessages<T>>::get(message_id);
                    match message.action {
                        Status::Withdraw => Self::add_pending_burn(message)?,
                        Status::Deposit => Self::add_pending_mint(message)?,
                        _ => (),
                    }
                }
                _ => (),
            }
            Self::update_status(message_id, Status::Pending, kind)
        }

        fn update_status(id: T::Hash, status: Status, kind: Kind) -> Result<(), &'static str> {
            match kind {
                Kind::Transfer => {
                    let mut message = <TransferMessages<T>>::get(id);
                    message.status = status;
                    <TransferMessages<T>>::insert(id, message);
                }
                Kind::Validator => {
                    let mut message = <ValidatorHistory<T>>::get(id);
                    message.status = status;
                    <ValidatorHistory<T>>::insert(id, message);
                }
                Kind::Bridge => {
                    let mut message = <BridgeMessages<T>>::get(id);
                    message.status = status;
                    <BridgeMessages<T>>::insert(id, message);
                }
                Kind::Limits => {
                    let mut message = <LimitMessages<T>>::get(id);
                    message.status = status;
                    <LimitMessages<T>>::insert(id, message);
                }
            }
            Ok(())
        }

        // needed because @message_id will be the same as initial
        fn reopen_for_burn_confirmation(message_id: T::Hash) -> Result<(), &'static str> {
            let message = <TransferMessages<T>>::get(message_id);
            let transfer_id = <TransferId<T>>::get(message_id);
            let mut transfer = <BridgeTransfers<T>>::get(transfer_id);
            let is_eth_response =
                message.status == Status::Confirmed || message.status == Status::Canceled;
            if !transfer.open && is_eth_response {
                transfer.votes = 0;
                transfer.open = true;
                <BridgeTransfers<T>>::insert(transfer_id, transfer);
                let validators = <ValidatorAccounts<T>>::get();
                validators
                    .iter()
                    .for_each(|a| <ValidatorVotes<T>>::insert((transfer_id, a.clone()), false));
            }
            Ok(())
        }

        fn check_validator(validator: T::AccountId) -> Result<(), &'static str> {
            let is_trusted = <Validators<T>>::contains_key(validator);
            ensure!(is_trusted, "Only validators can call this function");
            Ok(())
        }

        fn check_daily_account_volume(
            account: T::AccountId,
            amount: BalanceOf<T>,
        ) -> Result<(), &'static str> {
            let cur_pending = <DailyLimits<T>>::get(&account);
            let cur_pending_account_limit = <CurrentLimits<T>>::get().day_max_limit_for_one_address;
            let can_burn = cur_pending + amount < cur_pending_account_limit;

            //store current day (like 18768)
            let today = Self::get_day_pair().1;
            let user_blocked = <DailyBlocked<T>>::get(today).iter().any(|a| *a == account);

            if !can_burn {
                <DailyBlocked<T>>::mutate(today, |v| {
                    if !v.contains(&account) {
                        v.push(account.clone());
                        let now = <pallet_timestamp::Module<T>>::get();
                        let hash = (now.clone(), account.clone()).using_encoded(T::Hashing::hash);
                        Self::deposit_event(Event::AccountPausedMessage(hash, account, now))
                    }
                });
            }
            ensure!(
                can_burn && !user_blocked,
                "Transfer declined, user blocked due to daily volume limit."
            );

            Ok(())
        }

        fn check_amount(amount: BalanceOf<T>) -> Result<(), &'static str> {
            let max = <CurrentLimits<T>>::get().max_tx_value;
            let min = <CurrentLimits<T>>::get().min_tx_value;

            ensure!(
                amount >= min,
                "Invalid amount for transaction. Reached minimum limit."
            );
            ensure!(
                amount <= max,
                "Invalid amount for transaction. Reached maximum limit."
            );
            Ok(())
        }

        //open transactions check
        fn check_pending_burn(amount: BalanceOf<T>) -> Result<(), &'static str> {
            let new_pending_volume = <CurrentPendingBurn<T>>::get()
                .checked_add(&amount)
                .ok_or("Overflow adding to new pending burn volume")?;
            let can_burn = new_pending_volume < <CurrentLimits<T>>::get().max_pending_tx_limit;
            ensure!(can_burn, "Too many pending burn transactions.");
            Ok(())
        }

        fn check_pending_mint(amount: BalanceOf<T>) -> Result<(), &'static str> {
            let new_pending_volume = <CurrentPendingMint<T>>::get()
                .checked_add(&amount)
                .ok_or("Overflow adding to new pending mint volume")?;
            let can_burn = new_pending_volume < <CurrentLimits<T>>::get().max_pending_tx_limit;
            ensure!(can_burn, "Too many pending mint transactions.");
            Ok(())
        }

        fn check_limits(limits: &Limits<BalanceOf<T>>) -> Result<(), &'static str> {
            let max = BalanceOf::<T>::max_value();
            let min = BalanceOf::<T>::min_value();
            let passed = limits
                .into_array()
                .iter()
                .fold((true, true), |acc, l| match acc {
                    (true, true) => (l < &max, l > &min),
                    (true, false) => (l < &max, false),
                    (false, true) => (false, l > &min),
                    (_, _) => acc,
                });
            ensure!(passed.0, "Overflow setting limit");
            ensure!(passed.1, "Underflow setting limit");
            Ok(())
        }

        fn check_daily_holds(
            message: TransferMessage<T::AccountId, T::Hash, BalanceOf<T>>,
        ) -> Result<(), &'static str> {
            let from = message.substrate_address;
            let first_tx = <DailyHolds<T>>::get(from.clone());
            let daily_hold = T::BlocksPerEra::get();
            let day_passed = first_tx.0 + daily_hold < T::BlockNumber::from(0u32);

            if !day_passed {
                let account_balance = T::Currency::free_balance(&from);
                // 75% of potentially really big numbers
                let allowed_amount = account_balance
                    .checked_div(&BalanceOf::<T>::from(100u32))
                    .expect("Failed to calculate allowed withdraw amount")
                    .checked_mul(&BalanceOf::<T>::from(75u32))
                    .expect("Failed to calculate allowed withdraw amount");

                if message.amount > allowed_amount {
                    Self::update_status(message.message_id, Status::Canceled, Kind::Transfer)?;
                    fail!("Cannot withdraw more that 75% of first day deposit.");
                }
            }

            Ok(())
        }
    }

    impl<T: Config> BlockNumberProvider for Pallet<T> {
        type BlockNumber = T::BlockNumber;
        fn current_block_number() -> Self::BlockNumber {
            <frame_system::Module<T>>::block_number()
        }
    }

    // parse response of new_filter into a struct is hard in no_std, so use a
    // string matching to get the filter_id
    pub fn parse_new_eth_filter_response(resp_str: &str) -> Vec<u8> {
        if let Some(pos) = resp_str.find("result") {
            let start = pos + 9;
            let end = start + 46;
            let result = &resp_str[start..end];
            return result.as_bytes().to_vec();
        }
        vec![]
    }
}
