// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
use super::benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferKeepAliveBuilder};
use crate::{
    chain_spec, service,
    service::{db_config_dir, new_partial, ExecutorDispatch},
    Cli, Subcommand,
};
use fc_db::frontier_database_dir;
use frame_benchmarking_cli::ExtrinsicFactory;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use node_runtime::{Block, ExistentialDeposit, RuntimeApi};
use sc_cli::{ChainSpec, Result, RuntimeVersion, SubstrateCli};
use sc_service::{DatabaseSource, PartialComponents};
use sp_keyring::Sr25519Keyring;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Substrate Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/paritytech/substrate/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        let spec = match id {
            "" => {
                return Err(
                    "Please specify which chain you want to run, e.g. --dev or --chain=local"
                        .into(),
                )
            }
            "dev" => Box::new(chain_spec::development_config()),
            "local" => Box::new(chain_spec::local_testnet_config()),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        };
        Ok(spec)
    }

    fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &node_runtime::VERSION
    }
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        None => {
            let runner = cli.create_runner(&cli.run.base)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config, &cli).map_err(sc_cli::Error::Service)
            })
        }
        Some(Subcommand::Inspect(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|config| cmd.run::<Block, RuntimeApi, ExecutorDispatch>(config))
        }
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;

                runner.sync_run(|config| {
                    let PartialComponents {
                        client, ..
                    } = service::new_partial(&config, &cli)?;

                    // This switch needs to be in the client, since the client decides
                    // which sub-commands it wants to support.
                    match cmd {
                        BenchmarkCmd::Pallet(cmd) => {
                            if !cfg!(feature = "runtime-benchmarks") {
                                return Err(
                                    "Runtime benchmarking wasn't enabled when building the node. \
                                    You can enable it with `--features runtime-benchmarks`."
                                        .into(),
                                );
                            }

                            cmd.run::<Block, service::ExecutorDispatch>(config)
                        }
                        BenchmarkCmd::Block(cmd) => cmd.run(client),
                        #[cfg(not(feature = "runtime-benchmarks"))]
                        BenchmarkCmd::Storage(_) => Err(
                            "Storage benchmarking can be enabled with `--features runtime-benchmarks`."
                                .into(),
                        ),
                        #[cfg(feature = "runtime-benchmarks")]
                        BenchmarkCmd::Storage(cmd) => {
                            // ensure that we keep the task manager alive
                            let partial = new_partial(&config,&cli)?;
                            let db = partial.backend.expose_db();
                            let storage = partial.backend.expose_storage();

                            cmd.run(config, partial.client, db, storage)
                        },
                        BenchmarkCmd::Overhead(cmd) => {
                            let partial = new_partial(&config, &cli)?;
                            let ext_builder = RemarkBuilder::new(partial.client.clone());

                            cmd.run(
                                config,
                                client,
                                inherent_benchmark_data()?,
                                Vec::new(),
                                &ext_builder,
                            )
                        }
                        BenchmarkCmd::Extrinsic(cmd) => {
                            // ensure that we keep the task manager alive
                            let partial = service::new_partial(&config, &cli)?;
                            // Register the *Remark* and *TKA* builders.
                            let ext_factory = ExtrinsicFactory(vec![
                                Box::new(RemarkBuilder::new(partial.client.clone())),
                                Box::new(TransferKeepAliveBuilder::new(
                                    partial.client.clone(),
                                    Sr25519Keyring::Alice.to_account_id(),
                                    ExistentialDeposit::get(),
                                )),
                            ]);

                            cmd.run(
                                partial.client,
                                inherent_benchmark_data()?,
                                Vec::new(),
                                &ext_factory,
                            )
                        }
                        BenchmarkCmd::Machine(cmd) => {
                            cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
                        }
                    }
                })
            } else {
                Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
                    .into())
            }
        }
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::Sign(cmd)) => cmd.run(),
        Some(Subcommand::Verify(cmd)) => cmd.run(),
        Some(Subcommand::Vanity(cmd)) => cmd.run(),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = new_partial(&config, &cli)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = new_partial(&config, &cli)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = new_partial(&config, &cli)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = new_partial(&config, &cli)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                // Remove Frontier offchain db
                let db_config_dir = db_config_dir(&config);
                let frontier_database_config = match config.database {
                    DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
                        path: frontier_database_dir(&db_config_dir, "db"),
                        cache_size: 0,
                    },
                    DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
                        path: frontier_database_dir(&db_config_dir, "paritydb"),
                    },
                    _ => {
                        return Err(format!("Cannot purge `{:?}` database", config.database).into())
                    }
                };
                cmd.run(frontier_database_config)?;
                cmd.run(config.database)
            })
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = new_partial(&config, &cli)?;
                let aux_revert = Box::new(move |client, _, blocks| {
                    grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        }
        Some(Subcommand::FrontierDb(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                let PartialComponents { client, other, .. } = service::new_partial(&config, &cli)?;
                let frontier_backend = other.2;
                cmd.run::<_, node_runtime::opaque::Block>(client, frontier_backend)
            })
        }
    }
}
