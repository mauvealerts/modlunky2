use std::{io::BufRead, path::PathBuf};

use tokio::fs;
use toml::to_vec;

use crate::{
    data::{LoadMod, LoadOrderConfig, PlaylunkyConfig},
    Result,
};

const LOAD_ORDER_SUBPATH: &str = r"Mods\Packs\load_order.txt";
const PLAYLUNKY_CONFIG_SUBPATH: &str = r"playlunky.ini";

pub struct Playlunky {
    path: PathBuf,
}

impl Playlunky {
    pub fn new(install_dir: &str) -> Self {
        let path = PathBuf::new()
            .join(install_dir)
            .join(PLAYLUNKY_CONFIG_SUBPATH);
        Self { path }
    }

    pub async fn read(&self) -> Result<PlaylunkyConfig> {
        // TODO: handle missing dir
        let raw_bytes = fs::read(&self.path).await?;
        let config: PlaylunkyConfig = toml::de::from_slice(&raw_bytes[..])?;
        Ok(config)
    }

    pub async fn write(&self, config: PlaylunkyConfig) -> Result<()> {
        let bytes = to_vec(&config)?;
        Ok(fs::write(&self.path, bytes).await?)
    }
}

pub struct LoadOrder {
    path: PathBuf,
}

impl LoadOrder {
    pub fn new(install_dir: &str) -> Self {
        let path = PathBuf::new().join(install_dir).join(LOAD_ORDER_SUBPATH);
        Self { path }
    }

    pub async fn read(&self) -> Result<LoadOrderConfig> {
        // TODO: handle missing file
        let order = fs::read(&self.path).await?;
        order
            .lines()
            .into_iter()
            .map(|r| {
                r.map(|l| {
                    let enabled = !l.starts_with("--");
                    let id = if enabled { &l } else { &l[2..] }.to_string();
                    LoadMod { enabled, id }
                })
                .map_err(|e| e.into())
            })
            .collect()
    }

    pub async fn write(&self, order: LoadOrderConfig) -> Result<()> {
        // TODO: handle missing dir
        let text = order
            .into_iter()
            .map(|m| {
                let line = if m.enabled {
                    m.id
                } else {
                    "--".to_string() + &m.id
                };
                line + "\n"
            })
            .collect::<Vec<String>>()
            .concat();
        fs::write(&self.path, text).await?;
        Ok(())
    }
}
