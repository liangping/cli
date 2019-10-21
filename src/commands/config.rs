//! `config` subcommand - generate `libra-node` configuration

use abscissa_core::{Command, Options, Runnable};

/// `config` subcommand
#[derive(Command, Debug, Options)]
pub struct ConfigCmd {}

impl Runnable for ConfigCmd {
    fn run(&self) {}
}
