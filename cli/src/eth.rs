use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::{future, prelude::*};
// Substrate
use sc_client_api::BlockchainEvents;
use sc_network_sync::SyncingService;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
// Frontier
pub use fc_consensus::FrontierBlockImport;
use fc_rpc::{EthTask, OverrideHandle};
pub use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
// Local
use node_runtime::opaque::Block;

use crate::{
    service::{FullBackend, FullClient},
    EthConfiguration,
};

/// Frontier DB backend type.
pub type FrontierBackend = fc_db::Backend<Block>;

pub fn db_config_dir(config: &Configuration) -> PathBuf {
    config.base_path.config_dir(config.chain_spec.id())
}

pub struct FrontierPartialComponents {
    pub filter_pool: Option<FilterPool>,
    pub fee_history_cache: FeeHistoryCache,
    pub fee_history_cache_limit: FeeHistoryCacheLimit,
}

pub fn new_frontier_partial(
    config: &EthConfiguration,
) -> Result<FrontierPartialComponents, ServiceError> {
    Ok(FrontierPartialComponents {
        filter_pool: Some(Arc::new(Mutex::new(BTreeMap::new()))),
        fee_history_cache: Arc::new(Mutex::new(BTreeMap::new())),
        fee_history_cache_limit: config.fee_history_limit,
    })
}

/// A set of APIs that ethereum-compatible runtimes must implement.
pub trait EthCompatRuntimeApiCollection:
    sp_api::ApiExt<Block>
    + fp_rpc::ConvertTransactionRuntimeApi<Block>
    + fp_rpc::EthereumRuntimeRPCApi<Block>
{
}

impl<Api> EthCompatRuntimeApiCollection for Api where
    Api: sp_api::ApiExt<Block>
        + fp_rpc::ConvertTransactionRuntimeApi<Block>
        + fp_rpc::EthereumRuntimeRPCApi<Block>
{
}

pub fn spawn_frontier_tasks(
    task_manager: &TaskManager,
    client: Arc<FullClient>,
    backend: Arc<FullBackend>,
    frontier_backend: FrontierBackend,
    filter_pool: Option<FilterPool>,
    overrides: Arc<OverrideHandle<Block>>,
    fee_history_cache: FeeHistoryCache,
    fee_history_cache_limit: FeeHistoryCacheLimit,
    sync: Arc<SyncingService<Block>>,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<Block>,
        >,
    >,
) {
    // Spawn main mapping sync worker background task.

    let fc_db::Backend::KeyValue(b) = frontier_backend;
    task_manager.spawn_essential_handle().spawn(
        "frontier-mapping-sync-worker",
        Some("frontier"),
        fc_mapping_sync::kv::MappingSyncWorker::new(
            client.import_notification_stream(),
            Duration::new(6, 0),
            client.clone(),
            backend,
            overrides.clone(),
            Arc::new(b),
            3,
            0,
            fc_mapping_sync::SyncStrategy::Normal,
            sync,
            pubsub_notification_sinks,
        )
        .for_each(|()| future::ready(())),
    );

    // Spawn Frontier EthFilterApi maintenance task.
    if let Some(filter_pool) = filter_pool {
        // Each filter is allowed to stay in the pool for 100 blocks.
        const FILTER_RETAIN_THRESHOLD: u64 = 100;
        task_manager.spawn_essential_handle().spawn(
            "frontier-filter-pool",
            Some("frontier"),
            EthTask::filter_pool_task(client.clone(), filter_pool, FILTER_RETAIN_THRESHOLD),
        );
    }

    // Spawn Frontier FeeHistory cache maintenance task.
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("frontier"),
        EthTask::fee_history_task(
            client,
            overrides,
            fee_history_cache,
            fee_history_cache_limit,
        ),
    );
}
