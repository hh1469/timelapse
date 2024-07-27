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
        current_picture_folder: &Path,
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

        if !current_picture_folder.exists() {
            std::fs::create_dir_all(current_picture_folder)?;
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

        if let Err(e) = make_symlink(instance_name, &pic, current_picture_folder) {
            log::error!("{}", e);
        }

        Ok(())
    }
}

fn make_symlink(instance_name: &str, from: &Path, to: &Path) -> anyhow::Result<()> {
    let mut link = PathBuf::new();
    link.push(to);
    link.push(format!("{}.jpg", instance_name));

    if link.exists() {
        std::fs::remove_file(&link)?;
    }

    std::fs::soft_link(from, link)?;

    Ok(())
}
