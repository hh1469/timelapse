use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::anyhow;

use ureq::Agent;

#[derive(Debug)]
pub struct DownloadHandler {
    agent: Agent,
}

impl DownloadHandler {
    pub fn new() -> Self {
        let agent = ureq::builder()
            .timeout_connect(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .build();
        Self { agent }
    }

    pub fn make_picture(
        &self,
        instance_name: &str,
        index: u32,
        url: &str,
        path_to: &Path,
        snapshot_location: &Path,
    ) -> anyhow::Result<()> {
        log::info!("try to make picture with index {}", index);
        let mut reader = self.agent.get(url).call()?.into_reader();
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf)?;

        if buf.is_empty() {
            return Err(anyhow!("empty result"));
        }

        if !path_to.exists() {
            std::fs::create_dir_all(path_to)?;
        }

        let mut pic = PathBuf::new();
        pic.push(path_to);
        pic.push(format!("img-{:0>width$}.jpg", index, width = 8));

        let mut f = match pic.exists() {
            true => OpenOptions::new()
                .write(true)
                .append(false)
                .truncate(true)
                .open(pic.clone()),
            false => std::fs::File::create(pic.clone()),
        }?;

        f.write_all(&buf)?;
        f.flush()?;

        if let Err(e) = link_snapshot(instance_name, &pic, snapshot_location) {
            log::error!("link error: {}", e);
        }

        Ok(())
    }
}

fn link_snapshot(
    instance_name: &str,
    pic_from: &Path,
    snapshot_location: &Path,
) -> anyhow::Result<()> {
    log::warn!("1");
    if !snapshot_location.exists() {
        log::warn!("11");
        std::fs::create_dir_all(snapshot_location)?;
    }

    let mut link_to = PathBuf::new();
    link_to.push(snapshot_location);
    link_to.push(format!("{}.jpg", instance_name));

    log::warn!("2");
    if link_to.exists() {
        log::warn!("22");
        std::fs::remove_file(link_to.clone())?;
    }

    log::warn!("3");
    std::fs::soft_link(pic_from, link_to)?;
    log::warn!("4");

    Ok(())
}
