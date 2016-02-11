// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use util::*;
use ethcore::client::BlockChainClient;
use io::SyncIo;
use chain::SyncState;
use super::helpers::*;

#[test]
fn two_peers() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, false);
	net.peer_mut(2).chain.add_blocks(1000, false);
	net.sync();
	assert!(net.peer(0).chain.block_at(1000).is_some());
	assert_eq!(net.peer(0).chain.blocks.read().unwrap().deref(), net.peer(1).chain.blocks.read().unwrap().deref());
}

#[test]
fn status_after_sync() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, false);
	net.peer_mut(2).chain.add_blocks(1000, false);
	net.sync();
	let status = net.peer(0).sync.status();
	assert_eq!(status.state, SyncState::Idle);
}

#[test]
fn takes_few_steps() {
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(100, false);
	net.peer_mut(2).chain.add_blocks(100, false);
	let total_steps = net.sync();
	assert!(total_steps < 7);
}

#[test]
fn empty_blocks() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	for n in 0..200 {
		net.peer_mut(1).chain.add_blocks(5, n % 2 == 0);
		net.peer_mut(2).chain.add_blocks(5, n % 2 == 0);
	}
	net.sync();
	assert!(net.peer(0).chain.block_at(1000).is_some());
	assert_eq!(net.peer(0).chain.blocks.read().unwrap().deref(), net.peer(1).chain.blocks.read().unwrap().deref());
}

#[test]
fn forked() {
	::env_logger::init().ok();
	let mut net = TestNet::new(3);
	net.peer_mut(0).chain.add_blocks(300, false);
	net.peer_mut(1).chain.add_blocks(300, false);
	net.peer_mut(2).chain.add_blocks(300, false);
	net.peer_mut(0).chain.add_blocks(100, true); //fork
	net.peer_mut(1).chain.add_blocks(200, false);
	net.peer_mut(2).chain.add_blocks(200, false);
	net.peer_mut(1).chain.add_blocks(100, false); //fork between 1 and 2
	net.peer_mut(2).chain.add_blocks(10, true);
	// peer 1 has the best chain of 601 blocks
	let peer1_chain = net.peer(1).chain.numbers.read().unwrap().clone();
	net.sync();
	assert_eq!(net.peer(0).chain.numbers.read().unwrap().deref(), &peer1_chain);
	assert_eq!(net.peer(1).chain.numbers.read().unwrap().deref(), &peer1_chain);
	assert_eq!(net.peer(2).chain.numbers.read().unwrap().deref(), &peer1_chain);
}

#[test]
fn restart() {
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, false);
	net.peer_mut(2).chain.add_blocks(1000, false);

	net.sync_steps(8);

	// make sure that sync has actually happened
	assert!(net.peer(0).chain.chain_info().best_block_number > 100);
	net.restart_peer(0);

	let status = net.peer(0).sync.status();
	assert_eq!(status.state, SyncState::NotSynced);
}

#[test]
fn status_empty() {
	let net = TestNet::new(2);
	assert_eq!(net.peer(0).sync.status().state, SyncState::NotSynced);
}

#[test]
fn status_packet() {
	let mut net = TestNet::new(2);
	net.peer_mut(0).chain.add_blocks(1000, false);
	net.peer_mut(1).chain.add_blocks(1, false);

	net.start();

	net.sync_step_peer(0);

	assert_eq!(1, net.peer(0).queue.len());
	assert_eq!(0x00, net.peer(0).queue[0].packet_id);
}

#[test]
fn propagade_hashes() {
	let mut net = TestNet::new(3);
	net.peer_mut(1).chain.add_blocks(1000, false);
	net.peer_mut(2).chain.add_blocks(1000, false);
	net.sync();

	net.peer_mut(0).chain.add_blocks(10, false);
	net.sync_step_peer(0);

	// 2 peers to sync
	assert_eq!(2, net.peer(0).queue.len());
	// NEW_BLOCK_HASHES_PACKET
	assert_eq!(0x01, net.peer(0).queue[0].packet_id);
}

#[test]
fn propagade_blocks() {
	let mut net = TestNet::new(2);
	net.peer_mut(1).chain.add_blocks(10, false);
	net.sync();

	net.peer_mut(0).chain.add_blocks(10, false);
	net.trigger_block_verified(0);

	assert!(!net.peer(0).queue.is_empty());
	// NEW_BLOCK_PACKET
	assert_eq!(0x07, net.peer(0).queue[0].packet_id);
}
