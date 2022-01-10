// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
// This file is part of Frontier.
//
// Copyright (c) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

mod utils;

pub use sp_database::Database;

use codec::{Decode, Encode};
use fp_storage::PALLET_ETHEREUM_SCHEMA_CACHE;
use pallet_ethereum::EthereumStorageSchema;
use parking_lot::Mutex;
use sp_core::H256;
use sp_runtime::traits::Block as BlockT;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

const DB_HASH_LEN: usize = 32;
/// Hash type that this backend uses for the database.
pub type DbHash = [u8; DB_HASH_LEN];

/// Database settings.
pub struct DatabaseSettings {
    /// Where to find the database.
    pub source: DatabaseSettingsSrc,
}

/// Where to find the database.
#[derive(Debug, Clone)]
pub enum DatabaseSettingsSrc {
    /// Load a RocksDB database from a given path. Recommended for most uses.
    RocksDb {
        /// Path to the database.
        path: PathBuf,
        /// Cache size in MiB.
        cache_size: usize,
    },
}

impl DatabaseSettingsSrc {
    /// Return dabase path for databases that are on the disk.
    pub fn path(&self) -> Option<&Path> {
        match self {
            DatabaseSettingsSrc::RocksDb { path, .. } => Some(path.as_path()),
        }
    }
}

pub(crate) mod columns {
    pub const NUM_COLUMNS: u32 = 4;

    pub const META: u32 = 0;
    pub const BLOCK_MAPPING: u32 = 1;
    pub const TRANSACTION_MAPPING: u32 = 2;
    pub const SYNCED_MAPPING: u32 = 3;
}

pub(crate) mod static_keys {
    pub const CURRENT_SYNCING_TIPS: &[u8] = b"CURRENT_SYNCING_TIPS";
}

pub struct Backend<Block: BlockT> {
    meta: Arc<MetaDb<Block>>,
    mapping: Arc<MappingDb<Block>>,
}

impl<Block: BlockT> Backend<Block> {
    pub fn new(config: &DatabaseSettings) -> Result<Self, String> {
        let db = utils::open_database(config)?;

        Ok(Self {
            mapping: Arc::new(MappingDb {
                db: db.clone(),
                write_lock: Arc::new(Mutex::new(())),
                _marker: PhantomData,
            }),
            meta: Arc::new(MetaDb {
                db: db.clone(),
                _marker: PhantomData,
            }),
        })
    }

    pub fn mapping(&self) -> &Arc<MappingDb<Block>> {
        &self.mapping
    }

    pub fn meta(&self) -> &Arc<MetaDb<Block>> {
        &self.meta
    }
}

pub struct MetaDb<Block: BlockT> {
    db: Arc<dyn Database<DbHash>>,
    _marker: PhantomData<Block>,
}

impl<Block: BlockT> MetaDb<Block> {
    pub fn current_syncing_tips(&self) -> Result<Vec<Block::Hash>, String> {
        match self.db.get(
            crate::columns::META,
            &crate::static_keys::CURRENT_SYNCING_TIPS,
        ) {
            Some(raw) => {
                Ok(Vec::<Block::Hash>::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?)
            }
            None => Ok(Vec::new()),
        }
    }

    pub fn write_current_syncing_tips(&self, tips: Vec<Block::Hash>) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(
            crate::columns::META,
            crate::static_keys::CURRENT_SYNCING_TIPS,
            &tips.encode(),
        );

        self.db
            .commit(transaction)
            .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn ethereum_schema(&self) -> Result<Option<Vec<(EthereumStorageSchema, H256)>>, String> {
        match self
            .db
            .get(crate::columns::META, &PALLET_ETHEREUM_SCHEMA_CACHE.encode())
        {
            Some(raw) => Ok(Some(
                Decode::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?,
            )),
            None => Ok(None),
        }
    }

    pub fn write_ethereum_schema(
        &self,
        new_cache: Vec<(EthereumStorageSchema, H256)>,
    ) -> Result<(), String> {
        let mut transaction = sp_database::Transaction::new();

        transaction.set(
            crate::columns::META,
            &PALLET_ETHEREUM_SCHEMA_CACHE.encode(),
            &new_cache.encode(),
        );

        self.db
            .commit(transaction)
            .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }
}

pub struct MappingCommitment<Block: BlockT> {
    pub block_hash: Block::Hash,
    pub ethereum_block_hash: H256,
    pub ethereum_transaction_hashes: Vec<H256>,
}

#[derive(Clone, Encode, Decode)]
pub struct TransactionMetadata<Block: BlockT> {
    pub block_hash: Block::Hash,
    pub ethereum_block_hash: H256,
    pub ethereum_index: u32,
}

pub struct MappingDb<Block: BlockT> {
    db: Arc<dyn Database<DbHash>>,
    write_lock: Arc<Mutex<()>>,
    _marker: PhantomData<Block>,
}

impl<Block: BlockT> MappingDb<Block> {
    pub fn is_synced(&self, block_hash: &Block::Hash) -> Result<bool, String> {
        match self
            .db
            .get(crate::columns::SYNCED_MAPPING, &block_hash.encode())
        {
            Some(raw) => Ok(bool::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?),
            None => Ok(false),
        }
    }

    pub fn block_hash(&self, ethereum_block_hash: &H256) -> Result<Option<Block::Hash>, String> {
        match self
            .db
            .get(crate::columns::BLOCK_MAPPING, &ethereum_block_hash.encode())
        {
            Some(raw) => Ok(Some(
                Block::Hash::decode(&mut &raw[..]).map_err(|e| format!("{:?}", e))?,
            )),
            None => Ok(None),
        }
    }

    pub fn transaction_metadata(
        &self,
        ethereum_transaction_hash: &H256,
    ) -> Result<Vec<TransactionMetadata<Block>>, String> {
        match self.db.get(
            crate::columns::TRANSACTION_MAPPING,
            &ethereum_transaction_hash.encode(),
        ) {
            Some(raw) => Ok(Vec::<TransactionMetadata<Block>>::decode(&mut &raw[..])
                .map_err(|e| format!("{:?}", e))?),
            None => Ok(Vec::new()),
        }
    }

    pub fn write_none(&self, block_hash: Block::Hash) -> Result<(), String> {
        let _lock = self.write_lock.lock();

        let mut transaction = sp_database::Transaction::new();

        transaction.set(
            crate::columns::SYNCED_MAPPING,
            &block_hash.encode(),
            &true.encode(),
        );

        self.db
            .commit(transaction)
            .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }

    pub fn write_hashes(&self, commitment: MappingCommitment<Block>) -> Result<(), String> {
        let _lock = self.write_lock.lock();

        let mut transaction = sp_database::Transaction::new();

        transaction.set(
            crate::columns::BLOCK_MAPPING,
            &commitment.ethereum_block_hash.encode(),
            &commitment.block_hash.encode(),
        );

        for (i, ethereum_transaction_hash) in commitment
            .ethereum_transaction_hashes
            .into_iter()
            .enumerate()
        {
            let mut metadata = self.transaction_metadata(&ethereum_transaction_hash)?;
            metadata.push(TransactionMetadata::<Block> {
                block_hash: commitment.block_hash,
                ethereum_block_hash: commitment.ethereum_block_hash,
                ethereum_index: i as u32,
            });
            transaction.set(
                crate::columns::TRANSACTION_MAPPING,
                &ethereum_transaction_hash.encode(),
                &metadata.encode(),
            );
        }

        transaction.set(
            crate::columns::SYNCED_MAPPING,
            &commitment.block_hash.encode(),
            &true.encode(),
        );

        self.db
            .commit(transaction)
            .map_err(|e| format!("{:?}", e))?;

        Ok(())
    }
}
