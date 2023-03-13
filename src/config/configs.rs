use std::{collections::HashSet, error::Error, fs};

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct Cluster {
    pub servers: HashSet<String>,
    pub min_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct Configs {
    pub database: Database,
    pub server: Server,
    pub cluster: Cluster,
}

impl Configs {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let config = fs::read_to_string(path)?;
        let server_conf: Self = toml::from_str(&config)?;
        Ok(server_conf)
    }
}
