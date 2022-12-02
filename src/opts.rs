use std::{collections::HashSet, fs::File};

use serde::Deserialize;
use tracing_subscriber::EnvFilter;

#[derive(Deserialize)]
#[serde(tag = "version")]
pub enum Config {
    #[serde(rename = "1")]
    V1(ConfigV1),
}

#[derive(Clone, Deserialize)]
pub struct ConfigV1 {
    pub volumes: HashSet<String>,
}

#[derive(Clone)]
pub struct Opts {
    pub namespace: String,
    pub volumes: HashSet<String>,
}

impl Opts {
    pub fn new(namespace: String, config: ConfigV1) -> Self {
        Self {
            namespace,
            volumes: config.volumes,
        }
    }

    pub fn init_from_env() -> Result<Opts, anyhow::Error> {
        let filter = EnvFilter::from_default_env();
        tracing_subscriber::fmt().with_env_filter(filter).init();

        let namespace = std::env::var("PV_ASSEMBLER_NAMESPACE")?;

        let config: Config = serde_yaml::from_reader(File::open("/conf/pv-assembler.yaml")?)?;
        Ok(match config {
            Config::V1(config) => Opts::new(namespace, config),
        })
    }
}
