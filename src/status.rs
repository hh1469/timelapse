use anyhow::anyhow;
use std::{
    fs::OpenOptions,
    io::{Seek, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    index: u32,
    timestamp: i64,
    #[serde(skip_serializing, skip_deserializing)]
    file: Option<nix::fcntl::Flock<std::fs::File>>,
}

impl Status {
    pub fn new(timestamp: i64) -> Self {
        Status {
            index: 0,
            timestamp,
            file: Option::None,
        }
    }

    pub fn load(p: &Path) -> Option<Self> {
        if !p.exists() {
            log::debug!("path does not exists: {:?}", p);
            return None;
        }
        let content = match std::fs::read_to_string(p) {
            Ok(s) => s,
            Err(e) => {
                log::error!("{}", e);
                return None;
            }
        };
        let status: Status = match toml::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                log::error!("{}", e);
                return None;
            }
        };
        Some(status)
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn inc(&mut self) {
        self.index += 1;
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn lock(&mut self, p: &Path) -> anyhow::Result<()> {
        if let Some(parent) = p.parent() {
            if !p.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let f = match p.exists() {
            true => OpenOptions::new()
                .write(true)
                .append(false)
                .truncate(true)
                .open(p),
            false => std::fs::File::create(p),
        }?;
        let status_file = nix::fcntl::Flock::lock(f, nix::fcntl::FlockArg::LockExclusiveNonblock)
            .map_err(|e| anyhow!("flock error: {:?}", e))?;
        self.file = Some(status_file);
        Ok(())
    }

    pub fn store_to(&mut self) -> anyhow::Result<()> {
        let data = toml::to_string(self)?;
        if let Some(f) = self.file.as_mut() {
            f.set_len(0)?;
            f.seek(std::io::SeekFrom::Start(0))?;
            f.write_all(data.as_bytes())?;
            return Ok(());
        }
        Err(anyhow!("no status file"))
    }
}
