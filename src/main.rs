mod download_handler;
mod ffmpeg;
mod status;

use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use chrono::{DateTime, Datelike, Local};
use clap::Parser;

const SLEEP_INTERVAL: u64 = 25;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long)]
    name: String,
    #[arg(short, long)]
    url: String,
    #[arg(short, long)]
    picture_folder: PathBuf,
    #[arg(short, long)]
    video_folder: String,
    #[arg(short, long)]
    status_file: PathBuf,
    #[arg(short, long, default_value_t = 5)]
    interval: u64,
}

fn gen_video_name(name: &str) -> String {
    let t = chrono::Local::now();
    format!("{}_{}.mp4", name, t.format("%d_%m_%Y_%H_%M_%S"))
}

fn clean_folder(p: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_dir(p)?;
    for c in content {
        let p = c.map_err(|e| anyhow::anyhow!("{}", e))?.path();
        if !p.is_file() {
            continue;
        }

        if p.extension()
            .unwrap_or(std::ffi::OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .to_lowercase()
            .cmp(&"jpg".to_string())
            == Ordering::Equal
        {
            std::fs::remove_file(p)?;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    if !cli.picture_folder.exists() {
        std::fs::create_dir_all(&cli.picture_folder)?;
    }

    let video_folder = Path::new(&cli.video_folder);
    if !video_folder.exists() {
        std::fs::create_dir_all(video_folder)?;
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
        log::warn!("signal to finish the application");
    })
    .expect("unable to set ctrl+c handler");

    let mut date: DateTime<chrono::Local> = Local::now();

    let mut status = crate::status::Status::load(&cli.status_file)
        .unwrap_or(crate::status::Status::new(date.timestamp()));
    status.lock(&cli.status_file)?;

    let dh = crate::download_handler::DownloadHandler::new();

    let mut sleep = Duration::from_secs(0);
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // sleep and check exit status
        if sleep > Duration::from_millis(0) {
            std::thread::sleep(Duration::from_millis(SLEEP_INTERVAL));
            sleep = sleep
                .checked_sub(Duration::from_millis(SLEEP_INTERVAL))
                .unwrap_or(Duration::from_millis(0));
            continue;
        }

        let now: DateTime<chrono::Local> = Local::now();

        if now.year() != date.year() || now.month() != date.month() || now.day() != date.day() {
            log::info!("day changed {} {}", date, now);
            ffmpeg::make_timelapse(
                cli.picture_folder.to_str().unwrap_or(""),
                format!("{}/{}", &cli.video_folder, gen_video_name(&cli.name)).as_ref(),
            )?;
            clean_folder(&cli.picture_folder)?;
            status.reset();
            date = now;
        }

        let start = Instant::now();
        if let Err(e) = dh.make_picture(
            &cli.name,
            status.get_index(),
            &cli.url,
            &cli.picture_folder,
            video_folder,
        ) {
            log::error!("{}", e);
            sleep = Duration::from_secs(1);
            continue;
        }
        status.inc();
        status.store_to()?;

        if let Some(d) = Duration::from_secs(cli.interval).checked_sub(start.elapsed()) {
            sleep = d;
        } else {
            log::warn!("took to long");
        }
    }

    log::warn!("finished: {}", cli.name);

    Ok(())
}
