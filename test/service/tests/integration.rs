// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use cumulus_primitives::ParaId;
use cumulus_test_service::initial_head_data;
use futures::join;
use sc_service::TaskExecutor;
use substrate_test_runtime_client::AccountKeyring::*;

#[substrate_test_utils::test]
#[ignore]
async fn test_collating_and_non_collator_mode_catching_up(task_executor: TaskExecutor) {
	let para_id = ParaId::from(100);

	// start alice
	let alice = polkadot_test_service::run_test_node(task_executor.clone(), Alice, || {}, vec![]);

	// start bob
	let bob = polkadot_test_service::run_test_node(
		task_executor.clone(),
		Bob,
		|| {},
		vec![alice.addr.clone()],
	);

	// ensure alice and bob can produce blocks
	join!(alice.wait_for_blocks(2), bob.wait_for_blocks(2));

	// register parachain
	alice
		.register_para(
			para_id,
			cumulus_test_runtime::WASM_BINARY
				.expect("You need to build the WASM binary to run this test!")
				.to_vec()
				.into(),
			initial_head_data(para_id),
		)
		.await
		.unwrap();

	// run cumulus charlie (a validator)
	let charlie = cumulus_test_service::run_test_node(
		task_executor.clone(),
		Charlie,
		|| {},
		|| {},
		vec![],
		vec![alice.addr.clone(), bob.addr.clone()],
		para_id,
		true,
	);
	charlie.wait_for_blocks(2).await;

	// run cumulus dave (not a validator)
	//
	// a collator running in non-validator mode should be able to sync blocks from the tip of the
	// parachain
	let dave = cumulus_test_service::run_test_node(
		task_executor.clone(),
		Dave,
		|| {},
		|| {},
		vec![charlie.addr.clone()],
		vec![alice.addr.clone(), bob.addr.clone()],
		para_id,
		false,
	);
	dave.wait_for_blocks(4).await;

	join!(
		alice.task_manager.clean_shutdown(),
		bob.task_manager.clean_shutdown(),
		charlie.task_manager.clean_shutdown(),
		dave.task_manager.clean_shutdown(),
	);
}
