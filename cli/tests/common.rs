// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
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

#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use nix::sys::signal::{kill, Signal::SIGINT};
use nix::unistd::Pid;
use std::{convert::TryInto, process::Command};
use std::{
    path::Path,
    ops::{Deref, DerefMut},
    process::{Child, ExitStatus},
    thread,
    time::Duration,
};
use node_primitives::Block;
use remote_externalities::rpc_api;
use tokio::time::timeout;
static LOCALHOST_WS: &str = "ws://127.0.0.1:9944/";

/// Wait for the given `child` the given number of `secs`.
///
/// Returns the `Some(exit status)` or `None` if the process did not finish in the given time.
pub fn wait_for(child: &mut Child, secs: usize) -> Option<ExitStatus> {
    for i in 0..secs {
        match child.try_wait().unwrap() {
            Some(status) => {
                if i > 5 {
                    eprintln!("Child process took {} seconds to exit gracefully", i);
                }
                return Some(status);
            }
            None => thread::sleep(Duration::from_secs(1)),
        }
    }
    eprintln!("Took too long to exit (> {} seconds). Killing...", secs);
    let _ = child.kill();
    child.wait().unwrap();

    None
}

/// Run the node for a while (60 seconds)
pub fn run_dev_node_for_a_while(base_path: &Path) {
    let mut cmd = Command::new(cargo_bin("deeper-chain"));

    let mut cmd = cmd
        .args(&["--dev"])
        .arg("-d")
        .arg(base_path)
        .spawn()
        .unwrap();

    // Let it produce some blocks.
    thread::sleep(Duration::from_secs(60));
    assert!(
        cmd.try_wait().unwrap().is_none(),
        "the process should still be running"
    );

    // Stop the process
    kill(Pid::from_raw(cmd.id().try_into().unwrap()), SIGINT).unwrap();
    assert!(wait_for(&mut cmd, 40)
        .map(|x| x.success())
        .unwrap_or_default());
}

/// Wait for at least n blocks to be finalized within a specified time.
pub async fn wait_n_finalized_blocks(
	n: usize,
	timeout_secs: u64,
) -> Result<(), tokio::time::error::Elapsed> {
	timeout(Duration::from_secs(timeout_secs), wait_n_finalized_blocks_from(n, LOCALHOST_WS)).await
}

/// Wait for at least n blocks to be finalized from a specified node
pub async fn wait_n_finalized_blocks_from(n: usize, url: &str) {
	let mut built_blocks = std::collections::HashSet::new();
	let mut interval = tokio::time::interval(Duration::from_secs(2));

	loop {
		if let Ok(block) = rpc_api::get_finalized_head::<Block, _>(url.to_string()).await {
			built_blocks.insert(block);
			if built_blocks.len() > n {
				break
			}
		};
		interval.tick().await;
	}
}

pub struct KillChildOnDrop(pub Child);

impl Drop for KillChildOnDrop {
	fn drop(&mut self) {
		let _ = self.0.kill();
	}
}

impl Deref for KillChildOnDrop {
	type Target = Child;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for KillChildOnDrop {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}