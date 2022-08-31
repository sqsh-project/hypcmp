use clap::Parser;
use std::process::Command;

mod cli;
mod core;
mod util;

fn main() -> std::io::Result<()> {
    let config = cli::Cli::parse();
    // println!("{config:?}");

    util::is_git_dirty()?;

    let c = core::Benchmark::from_config(config.config)?;
    // println!("{c}");

    let dir = tempfile::tempdir()?;
    let mut files_to_be_merged: Vec<String> = Vec::new();
    let current_branch = util::get_current_branch_or_id()?;
    // println!("CB: {current_branch:?}");
    for (label, run) in c.run.iter() {
        let mut cmd = Command::new("hyperfine");
        cmd.args(c.to_hyperfine_params());

        let mut json = vec!["--export-json".to_string()];
        let mut filename = label.clone();
        filename.push_str(".json");
        let output = dir.path().join(filename).display().to_string();
        json.push(output.clone());
        cmd.args(json);

        cmd.args(run.to_hyperfine_params());
        println!("Running: {cmd:?}");
        cmd.output()?; // TODO: Catch possible errors
        files_to_be_merged.push(output);
    }
    let json = util::merge_json_files(&files_to_be_merged)?;
    util::write_json_to_disk(json, &c.output)?;
    util::cleanup(files_to_be_merged, dir)?;
    util::checkout(current_branch)?;
    Ok(())
}
