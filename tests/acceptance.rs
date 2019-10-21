//! Acceptance test: runs the application as a subprocess and asserts its
//! output for given argument combinations matches what is expected.
//!
//! For more information, see:
//! <https://docs.rs/abscissa_core/latest/abscissa_core/testing/index.html>

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

use abscissa_core::testing::prelude::*;
use lazy_static::lazy_static;
use libra_config::config::{ConsensusConfig, NetworkConfig, NodeConfig, PersistableConfig};
use open_libra::peer_info::PeerInfo;
use tempfile::tempdir;

lazy_static! {
    /// Executes your application binary via `cargo run`.
    ///
    /// Storing this value in a `lazy_static!` ensures that all instances of
    /// the runner acquire a mutex when executing commands and inspecting
    /// exit statuses, serializing what would otherwise be multithreaded
    /// invocations as `cargo test` executes tests in parallel by default.
    pub static ref RUNNER: CmdRunner = CmdRunner::default();
}

#[test]
fn config_generator() {
    let mut runner = RUNNER.clone();

    let dir = tempdir().unwrap();
    let cmd = runner
        .arg("config")
        .arg("-o")
        .arg(dir.path())
        .capture_stdout()
        .run();

    cmd.wait().unwrap().expect_success();

    // Make sure the generated config files load
    ConsensusConfig::load_config(dir.path().join("consensus_keypair.config.toml"));
    NetworkConfig::load_config(dir.path().join("network_keypairs.config.toml"));
    NodeConfig::load_config(dir.path().join("node.config.toml"));
    PeerInfo::load_config(dir.path().join("peer_info.toml"));
}
