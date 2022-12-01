use std::{collections::{HashMap, HashSet}, fs::File};

use serde::Deserialize;
use tracing_subscriber::EnvFilter;

#[derive(Deserialize)]
#[serde(tag = "version")]
pub enum Config {
    #[serde(rename = "1")]
    V1(ConfigV1)
}

#[derive(Clone, Deserialize)]
pub struct ConfigV1 {
    pub volumes: HashMap<String, VolumeConfig>,
}

#[derive(Clone, Deserialize)]
pub struct VolumeConfig {
    pub allowed_namespaces: HashSet<String>,
}

pub struct Opts {}

impl Opts {
    pub fn init_from_env() -> Result<ConfigV1, anyhow::Error> {
        let filter = EnvFilter::from_default_env();
        tracing_subscriber::fmt().with_env_filter(filter).init();
    
        let config: Config = serde_yaml::from_reader(File::open("/conf/pv-assembler.yaml")?)?;
        Ok(match config {
            Config::V1(config) => config
        })
    }
}
