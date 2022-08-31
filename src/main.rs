use clap::Parser;
use serde::Deserialize;
use std::{
    collections::HashMap, fmt::Display, fs::File, io::Read, path::PathBuf, process::Command,
};
mod cli;
mod util;

fn main() -> std::io::Result<()> {
    let config = cli::Cli::parse();
    // println!("{config:?}");

    util::is_git_dirty()?;

    let c = Benchmark::from_config(config.config)?;
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
        cmd.output().expect("Failed"); // TODO: Catch possible errors
        files_to_be_merged.push(output);
    }
    let json = util::merge_json_files(&files_to_be_merged)?;
    util::write_json_to_disk(json, &c.output)?;
    util::cleanup(files_to_be_merged, dir)?;
    util::checkout(current_branch)?;
    Ok(())
}

#[derive(Deserialize, Debug)]
struct Benchmark {
    output: String,
    hyperfine_params: Vec<String>,
    run: HashMap<String, Run>,
}

impl Display for Benchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Common Settings:")?;
        writeln!(f, "{:?}", self.to_hyperfine_params())?;
        for (k, v) in self.run.iter() {
            writeln!(f, "Subcommand Settings: {k:?}")?;
            writeln!(f, "{:?}", v.to_hyperfine_params())?;
        }
        writeln!(f)
    }
}

impl Benchmark {
    fn from_config(config: PathBuf) -> std::io::Result<Self> {
        let mut f = File::open(config)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        let value = content.as_str();
        let result = toml::from_str(value)?;
        Ok(result)
    }
    fn to_hyperfine_params(&self) -> Vec<String> {
        self.hyperfine_params.clone()
    }
}

#[derive(Deserialize, Debug)]
struct Run {
    commits: Option<Vec<String>>,
    cleanup: Option<String>,
    prepare: Option<String>,
    setup: Option<String>,
    command: String,
}

impl Run {
    fn to_hyperfine_params(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        match &self.commits {
            Some(ids) => {
                result.push("--parameter-list".to_string());
                result.push("commit".to_string());
                result.push(ids.join(","));
            }
            None => (),
        }
        match &self.cleanup {
            Some(cmd) => {
                result.push("--cleanup".to_string());
                result.push(cmd.clone());
            }
            None => (),
        }
        match &self.prepare {
            Some(cmd) => {
                result.push("--prepare".to_string());
                result.push(cmd.clone());
            }
            None => (),
        }
        match (&self.setup, &self.commits) {
            (Some(scmd), Some(_)) => {
                result.push("--setup".to_string());
                let concat = format!("git checkout {{commit}} && {scmd}");
                result.push(concat);
            }
            (None, Some(_)) => {
                result.push("--setup".to_string());
                result.push("git checkout {commit}".to_string());
            }
            _ => (),
        }
        result.push(self.command.clone());
        result
    }
}
