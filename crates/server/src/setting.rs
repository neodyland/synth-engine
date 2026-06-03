use std::collections::HashMap;

use serde::Deserialize;
use tokio::fs::read;

fn default_bind_addr() -> String {
    "[::]:3000".to_string()
}

fn default_cache_size() -> usize {
    200
}

#[derive(Deserialize)]
pub struct SettingModel {
    pub path: String,
}

#[derive(Deserialize)]
pub struct Setting {
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
    pub models: HashMap<String, SettingModel>,
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
}

impl Setting {
    pub async fn load(file: &str) -> anyhow::Result<Self> {
        Ok(toml::from_slice(&read(file).await?)?)
    }
}
