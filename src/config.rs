//! OpenLibra CLI Config

use abscissa_core::Config;
use serde::{Deserialize, Serialize};

/// OpenLibra Configuration
#[derive(Clone, Config, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {}
