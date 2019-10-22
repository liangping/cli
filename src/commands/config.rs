//! `config` subcommand - generate `libra-node` configuration

pub mod builder;

use self::builder::Builder;
use crate::prelude::*;
use abscissa_core::{Command, Options, Runnable};
use std::path::PathBuf;

/// `config` subcommand
#[derive(Command, Debug, Options)]
pub struct ConfigCmd {
    /// Directory where config files will be output to
    #[options(short = "o", long = "output", help = "output directory")]
    output_dir: Option<PathBuf>,
}

impl Runnable for ConfigCmd {
    fn run(&self) {
        let mut builder = Builder::new();

        if let Some(output_dir) = &self.output_dir {
            builder.with_output_dir(output_dir);
        }

        builder.build().unwrap();

        status_ok!("Success", "all configuration files generated successfully");
        status_info!(
            "Visit",
            "https://github.com/open-libra/devnet to join OpenLibra Devnet"
        );
    }
}
