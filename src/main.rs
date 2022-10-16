use clap::Parser;
use log::{debug, error, info, trace};
use std::process::Command;

mod cli;
mod core;
mod util;

fn main() -> std::io::Result<()> {
    let config = cli::Cli::parse();
    env_logger::Builder::new()
        .filter_level(config.verbose.log_level_filter())
        .init();
    debug!("Loaded configuration: {config:?}");

    util::hyperfine_installed()?;
    util::is_git_dirty()?;

    let c = core::Benchmark::from_config(config.config)?;
    trace!("Benchmark Setup: {c:#?}");

    let dir = tempfile::tempdir()?;
    debug!("Temporary Directory: {dir:?}");

    let mut files_to_be_merged: Vec<String> = Vec::new();
    let current_branch = util::get_current_branch_or_id()?;
    info!("Current branch is: {current_branch:?}");

    for (label, run) in c.run.iter() {
        debug!("Run: {run:?}");

        let mut cmd = Command::new("hyperfine");
        cmd.args(c.to_hyperfine_params());

        let mut json = vec!["--export-json".to_string()];
        let mut filename = label.clone();
        filename.push_str(".json");
        let output = dir.path().join(filename).display().to_string();
        json.push(output.clone());
        cmd.args(json);

        cmd.args(run.to_hyperfine_params());
        info!("Running: {cmd:?}");

        let result = cmd.output()?;
        if result.status.success() {
            debug!("Benchmark run successful");
            files_to_be_merged.push(output);
        } else {
            let msg = result.stderr;
            let s = util::to_string(msg);
            error!("Run {label:?} failed. Skipping...");
            error!("Hyperfine message:   {}", s);
            error!("Run parameters were: {cmd:?}");
        }
    }
    if files_to_be_merged.is_empty() {
        let msg = "No hyperfine benchmark run";
        error!("{msg}");
        let err = std::io::Error::new(std::io::ErrorKind::Other, msg);
        util::checkout(current_branch)?;
        return Err(err)
    } else {
        let json = util::merge_json_files(&files_to_be_merged)?;
        util::export_to_html(&json)?;
        util::write_json_to_disk(json, &c.output)?;
        util::cleanup(files_to_be_merged, dir)?;
        util::checkout(current_branch)?;
    }
    Ok(())
}
