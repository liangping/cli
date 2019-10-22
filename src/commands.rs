//! OpenLibra Subcommands

pub mod config;
#[cfg(feature = "genesis")]
pub mod genesis;
mod version;

#[cfg(feature = "genesis")]
use self::genesis::GenesisCmd;
use self::{config::ConfigCmd, version::VersionCmd};
use crate::config::AppConfig;
use abscissa_core::{Command, Configurable, Help, Options, Runnable};
use std::path::PathBuf;

/// OpenLibra Configuration Filename
pub const CONFIG_FILE: &str = "open-libra.toml";

/// OpenLibra Subcommands
#[derive(Command, Debug, Options, Runnable)]
pub enum OpenLibraCmd {
    /// The `config` subcommand
    #[options(help = "generate libra-node configuration")]
    Config(ConfigCmd),

    /// The `genesis` subcommand
    #[cfg(feature = "genesis")]
    #[options(help = "create genesis.blob from input peer_info.toml files")]
    Genesis(GenesisCmd),

    /// The `help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// The `version` subcommand
    #[options(help = "display version information")]
    Version(VersionCmd),
}

impl Configurable<AppConfig> for OpenLibraCmd {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        // Check if the config file exists, and if it does not, ignore it.
        let filename = PathBuf::from(CONFIG_FILE);

        if filename.exists() {
            Some(filename)
        } else {
            None
        }
    }
}
