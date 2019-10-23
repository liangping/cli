//! Libra configuration builder.
//!
//! Parts adapted from upstream Libra's `SwarmConfigBuilder`:
//! <https://github.com/libra/libra/blob/master/config/config-builder/src/swarm_config.rs>

use crate::{
    error::Error,
    peer_info::{self, PeerInfo},
    prelude::*,
};
use libra_config::{
    config::{ConsensusConfig, NetworkConfig, NodeConfig, PersistableConfig, RoleType},
    keys::{ConsensusKeyPair, NetworkKeyPairs},
    trusted_peers::{
        ConfigHelpers, ConsensusPeersConfig, ConsensusPrivateKey, NetworkPeersConfig,
        NetworkPrivateKeys,
    },
};
use parity_multiaddr::Multiaddr;
use std::path::{Path, PathBuf};
use std::fs;

/// Default address to listen on
pub const DEFAULT_LISTEN_ADDRESS: &str = "/ip4/127.0.0.1";

/// Libra configuration builder
pub struct Builder {
    /// Output directory
    output_dir: PathBuf,

    /// Address to listen on
    listen_address: Multiaddr,

    /// Address to advertise to the network
    advertised_address: Multiaddr,

    /// Seed to use when generating keys (default random)
    key_seed: Option<[u8; 32]>,

    /// Node `RoleType` (either `Validator` or `FullNode`)
    role: RoleType,

    /// Is this network permissioned?
    is_permissioned: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            listen_address: DEFAULT_LISTEN_ADDRESS.parse().unwrap(),
            advertised_address: DEFAULT_LISTEN_ADDRESS.parse().unwrap(),
            key_seed: None,
            role: RoleType::Validator,
            is_permissioned: true, // TODO(tarcieri): set this to false
        }
    }
}

impl Builder {
    /// Create a new config builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set output directory
    pub fn with_output_dir(&mut self, output_dir: impl AsRef<Path>) -> &mut Self {
        self.output_dir = output_dir.as_ref().to_path_buf();
        self
    }

    /// Set listen address
    pub fn with_listen_address(&mut self, listen_address: impl ToString) -> &mut Self {
        self.listen_address = listen_address.to_string().parse().unwrap();
        self
    }

    /// Set advertised address
    pub fn with_advertised_address(&mut self, advertised_address: impl ToString) -> &mut Self {
        self.advertised_address = advertised_address.to_string().parse().unwrap();
        self
    }

    /// Set key seed value (default secure random)
    pub fn with_key_seed(&mut self, seed: [u8; 32]) -> &mut Self {
        self.key_seed = Some(seed);
        self
    }

    /// Configure whether or not the network is permissioned
    pub fn with_is_permissioned(&mut self, is_permissioned: bool) -> &mut Self {
        // TODO(tarcieri): support permissionless networks
        assert!(
            !is_permissioned,
            "support for `is_permissioned: false` unimplemented"
        );
        self
    }

    /// Build the configuration, writing the output to `output_dir`
    pub fn build(self) -> Result<NodeConfig, Error> {
        assert_eq!(
            self.role,
            RoleType::Validator,
            "only validator role is presently supported"
        );

        fs::create_dir_all(self.output_dir.as_path()).expect("Can not create output directory");

        // Use the OS RNG to generate a seed unless one has been explicitly provided
        let key_seed = self.key_seed.unwrap_or_else(|| {
            let mut s = [0u8; 32];
            getrandom::getrandom(&mut s).expect("RNG failure!");
            s
        });

        // Generate private keys as well as consensus and network configs
        let (private_keys, consensus_peers_config, network_peers_config) =
            ConfigHelpers::gen_validator_nodes(1, Some(key_seed));

        let peer_id = network_peers_config.peers.keys().next().unwrap().to_owned();

        self.generate_peer_info(&peer_id, consensus_peers_config, network_peers_config);

        let (_account, (consensus_private_key, network_private_keys)) =
            private_keys.into_iter().next().unwrap();

        let consensus_config = self.generate_consensus_config(consensus_private_key);
        let network_config = self.generate_network_config(&peer_id, network_private_keys);

        let node_config = NodeConfig {
            base: Default::default(),
            networks: vec![network_config],
            consensus: consensus_config,
            metrics: Default::default(),
            execution: Default::default(),
            admission_control: Default::default(),
            debug_interface: Default::default(),
            storage: Default::default(),
            mempool: Default::default(),
            state_sync: Default::default(),
            log_collector: Default::default(),
            vm_config: Default::default(),
            secret_service: Default::default(),
        };

        let node_config_file = self.output_dir.join("node.config.toml");
        node_config.save_config(&node_config_file);
        status_ok!("Generated", "{}", node_config_file.display());

        Ok(node_config)
    }

    /// Generate `ConsensusConfig` and write `consensus_keypair.config.toml`
    fn generate_consensus_config(&self, private_key: ConsensusPrivateKey) -> ConsensusConfig {
        let consensus_config = ConsensusConfig::default();

        let consensus_keypair = ConsensusKeyPair::load(Some(private_key.consensus_private_key));
        let consensus_keypair_file = self
            .output_dir
            .join(&consensus_config.consensus_keypair_file);

        consensus_keypair.save_config(&consensus_keypair_file);
        status_ok!("Generated", "{}", consensus_keypair_file.display());

        consensus_config
    }

    /// Generate `NetworkConfig` and write `network_keypairs.config.toml`
    fn generate_network_config(
        &self,
        peer_id: &str,
        private_keys: NetworkPrivateKeys,
    ) -> NetworkConfig {
        let network_keypairs = NetworkKeyPairs::load(
            private_keys.network_signing_private_key,
            private_keys.network_identity_private_key,
        );

        let mut network_config = NetworkConfig::default();
        network_config.peer_id = peer_id.to_owned();

        network_config.role = match self.role {
            RoleType::Validator => "validator",
            RoleType::FullNode => "full_node",
        }
        .to_owned();

        network_config.listen_address = self.listen_address.clone();
        network_config.advertised_address = self.advertised_address.clone();
        network_config.is_permissioned = self.is_permissioned;

        let network_keypairs_file = self.output_dir.join(&network_config.network_keypairs_file);
        network_keypairs.save_config(&network_keypairs_file);
        status_ok!("Generated", "{}", network_keypairs_file.display());

        network_config
    }

    /// Generate `PeerInfo` and write `peer_info.toml`
    fn generate_peer_info(
        &self,
        peer_id: &str,
        consensus_peers: ConsensusPeersConfig,
        network_peers: NetworkPeersConfig,
    ) -> PeerInfo {
        let consensus_info = consensus_peers.peers.into_iter().next().unwrap().1;
        let network_info = network_peers.peers.into_iter().next().unwrap().1;
        let peer_info = PeerInfo::new(peer_id, consensus_info, network_info);

        let peer_info_file = self.output_dir.join(peer_info::DEFAULT_FILENAME);
        peer_info.save_config(&peer_info_file);
        status_ok!("Generated", "{}", peer_info_file.display());

        peer_info
    }
}
