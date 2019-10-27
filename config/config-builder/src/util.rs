// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::{
    config::{NodeConfig, NodeConfigHelpers},
    trusted_peers::{
        ConfigHelpers, ConsensusPeersConfig, ConsensusPrivateKey, NetworkPeersConfig,
        NetworkPrivateKeys,
    },
};
use libra_crypto::{ed25519::*, test_utils::KeyPair};
use libra_prost_ext::MessageExt;
use libra_types::{account_address::AccountAddress, transaction::SignatureCheckedTransaction};
use rand::{Rng, SeedableRng};
use std::{collections::HashMap, fs::File, io::prelude::*};
use vm_genesis::encode_genesis_transaction_with_validator;

pub fn gen_genesis_transaction(
    faucet_account_keypair: &KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    consensus_peers_config: &ConsensusPeersConfig,
    network_peers_config: &NetworkPeersConfig,
) -> SignatureCheckedTransaction {
    encode_genesis_transaction_with_validator(
        &faucet_account_keypair.private_key,
        faucet_account_keypair.public_key.clone(),
        consensus_peers_config.get_validator_set(network_peers_config),
    )
}

pub fn gen_genesis_transaction_bytes(
    faucet_account_keypair: &KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    consensus_peers_config: &ConsensusPeersConfig,
    network_peers_config: &NetworkPeersConfig,
) -> Vec<u8> {
    let genesis_transaction = gen_genesis_transaction(
        faucet_account_keypair,
        consensus_peers_config,
        network_peers_config,
    );
    let genesis_transaction: libra_types::proto::types::SignedTransaction =
        genesis_transaction.into();
    genesis_transaction.to_vec().unwrap()
}

/// Returns the validator keys, consensus peers, config, and the genesis keyapir
pub fn get_test_config_with_validators() -> (
    HashMap<AccountAddress, (ConsensusPrivateKey, NetworkPrivateKeys)>,
    ConsensusPeersConfig,
    NodeConfig,
    KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
) {
    // TODO: test config should be moved here instead of config crate
    let config = NodeConfigHelpers::get_single_node_test_config(true);
    // Those configs should be different on every call. We bypass the
    // costly StdRng initialization
    let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng = rand::rngs::StdRng::from_seed(seed_buf);
    let (private_key, _) = compat::generate_keypair(&mut rng);
    let keypair = KeyPair::from(private_key);
    let (validator_keys, test_consensus_peers, test_network_peers) =
        ConfigHelpers::gen_validator_nodes(1, None);
    let genesis_transaction =
        gen_genesis_transaction_bytes(&keypair, &test_consensus_peers, &test_network_peers);
    let mut genesis_transaction_file = File::create(config.get_genesis_transaction_file())
        .expect("[config] Failed to create file for storing genesis transaction");
    genesis_transaction_file
        .write_all(&genesis_transaction)
        .expect("[config] Failed to write genesis txn to file");
    (validator_keys, test_consensus_peers, config, keypair)
}

/// Returns the config as well as the genesis keyapir
pub fn get_test_config() -> (NodeConfig, KeyPair<Ed25519PrivateKey, Ed25519PublicKey>) {
    let (_validator_keys, _test_consensus_peers, config, keypair) =
        get_test_config_with_validators();
    (config, keypair)
}
