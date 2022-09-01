use clap::Parser;
use log::{debug, warn, info};
use std::process::Command;

mod cli;
mod core;
mod util;

fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = cli::Cli::parse();
    debug!("Loaded configuration: {config:?}");

    util::is_git_dirty()?;
    debug!("Git state: clean");

    let c = core::Benchmark::from_config(config.config)?;
    debug!("Benchmark Setup: {c:?}");

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
            warn!("Run[{label:?}] failed with '{s:?}'");
            if s.contains("The setup command terminated with a non-zero exit") {
                eprintln!("[Warning] Run {label:?} failed. Setup failed");
            }
            eprintln!("[Warning] Run {label:?} failed. Skipping... -> Hyperfine: {s:?}");
            eprintln!("[Warning] {cmd:?}");
        }
    }
    let json = util::merge_json_files(&files_to_be_merged)?;
    util::write_json_to_disk(json, &c.output)?;
    util::cleanup(files_to_be_merged, dir)?;
    util::checkout(current_branch)?;
    Ok(())
}
