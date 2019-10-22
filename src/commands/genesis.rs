//! `genesis` subcommand - generate `libra-node` configuration

use crate::{peer_info::PeerInfo, prelude::*};
use abscissa_core::{Command, Options, Runnable};
use libra_config::{
    config::PersistableConfig,
    trusted_peers::{ConsensusPeersConfig, NetworkPeersConfig},
};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use libra_types::{proto::types::SignedTransaction, validator_set::ValidatorSet};
use std::{
    convert::TryFrom,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::exit,
};

/// Ed25519 keypairs. This is how `mint.key` is serialized
pub type Ed25519KeyPair = KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;

/// `genesis` subcommand
#[derive(Command, Debug, Options)]
pub struct GenesisCmd {
    /// Directory where genesis files will be output to
    #[options(short = "o", long = "output", help = "output directory")]
    output_dir: Option<PathBuf>,

    /// Paths to `peer_info.toml`-formatted files
    #[options(free, help = "peer_info.toml-formatted files")]
    peer_info_files: Vec<PathBuf>,
}

impl Runnable for GenesisCmd {
    fn run(&self) {
        let mut consensus_peers_config = ConsensusPeersConfig::default();
        let mut network_peers_config = NetworkPeersConfig::default();

        for peer_info in self.peer_info_files.iter().map(PeerInfo::load_config) {
            let mut dup = consensus_peers_config
                .peers
                .insert(peer_info.id.clone(), peer_info.consensus)
                .is_some();

            dup |= network_peers_config
                .peers
                .insert(peer_info.id.clone(), peer_info.network)
                .is_some();

            if dup {
                status_err!("duplicate peer ID: {}", &peer_info.id);
                exit(1);
            }
        }

        if consensus_peers_config.peers.is_empty() || network_peers_config.peers.is_empty() {
            status_err!("no peers given! Usage: open-libra genesis peer1.toml peer2.toml ...");
            exit(1);
        }

        let validator_set = consensus_peers_config.get_validator_set(&network_peers_config);

        self.create_genesis_transaction(validator_set)
            .unwrap_or_else(|e| {
                status_err!("error creating genesis.blob: {}", e);
                exit(1);
            });

        let output_dir = self.output_dir_or_default();

        let consensus_peers_file = output_dir.join("consensus_peers.config.toml");
        consensus_peers_config.save_config(&consensus_peers_file);
        status_ok!("Generated", "{}", consensus_peers_file.display());

        let network_peers_file = output_dir.join("network_peers.config.toml");
        network_peers_config.save_config(&network_peers_file);
        status_ok!("Generated", "{}", network_peers_file.display());
    }
}

impl GenesisCmd {
    /// Create the genesis transaction
    fn create_genesis_transaction(
        &self,
        validator_set: ValidatorSet,
    ) -> Result<SignedTransaction, io::Error> {
        let faucet_keypair = self.faucet_keypair()?;

        let genesis_transaction =
            SignedTransaction::from(vm_genesis::encode_genesis_transaction_with_validator(
                &faucet_keypair.private_key,
                faucet_keypair.public_key.clone(),
                validator_set,
            ));

        let genesis_transaction_file = self.output_dir_or_default().join("genesis.blob");
        File::create(&genesis_transaction_file)?.write_all(&genesis_transaction.signed_txn)?;
        status_ok!("Generated", "{}", genesis_transaction_file.display());

        Ok(genesis_transaction)
    }

    /// Load the faucet private key
    // TODO(tarcieri): support for loading an existing faucet key from a file
    fn faucet_keypair(&self) -> Result<Ed25519KeyPair, io::Error> {
        let mut key_bytes = [0u8; 32];
        getrandom::getrandom(&mut key_bytes).expect("RNG failure!");
        let key = Ed25519PrivateKey::try_from(key_bytes.as_ref()).unwrap();
        let keypair = Ed25519KeyPair::from(key);

        let mint_key_file = self.output_dir_or_default().join("mint.key");
        File::create(&mint_key_file)?.write_all(&bincode::serialize(&keypair).unwrap())?;
        status_ok!("Generated", "{}", mint_key_file.display());

        Ok(keypair)
    }

    /// Get the output directory or the default
    fn output_dir_or_default(&self) -> &Path {
        self.output_dir
            .as_ref()
            .map(AsRef::as_ref)
            .unwrap_or_else(|| Path::new("."))
    }
}
