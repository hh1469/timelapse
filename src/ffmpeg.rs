pub fn make_timelapse(from: &str, to: &str) -> anyhow::Result<()> {
    log::debug!("to: {}", to);
    let glob = format!("{}/*.jpg", from);
    let args = [
        "-y",
        "-framerate",
        "30",
        "-pattern_type",
        "glob",
        "-i",
        glob.as_ref(),
        "-s:v",
        "640x320",
        "-c:v",
        "libx264",
        "-crf",
        "17",
        "-pix_fmt",
        "yuv420p",
        to,
    ];
    let child = std::process::Command::new("ffmpeg").args(args).spawn()?;
    let output = child.wait_with_output()?;
    log::trace!("{:?}", output);
    Ok(())
}
