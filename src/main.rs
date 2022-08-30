use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
    process::Command,
};
use tempfile::TempDir;
mod cli;
use clap::Parser;

fn main() -> std::io::Result<()> {
    let config = cli::Cli::parse();
    println!("{config:?}");

    let c = Benchmark::from_config(config.config)?;
    println!("{c}");

    let dir = tempfile::tempdir()?;
    let mut files_to_be_merged: Vec<String> = Vec::new();

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
        println!("{cmd:?}");
        cmd.status().expect("Failed"); // Execute command back to back
        files_to_be_merged.push(output);
    }
    let json = merge_json_files(&files_to_be_merged)?;
    write_json_to_disk(json, &c.output)?;
    cleanup(files_to_be_merged, dir)?;
    Ok(())
}

fn cleanup(tempfilelist: Vec<String>, dir: TempDir) -> std::io::Result<()> {
    for file in tempfilelist {
        drop(file)
    }
    dir.close()
}

fn write_json_to_disk(json: Value, output: &String) -> std::io::Result<()> {
    let json_pp = serde_json::to_string_pretty(&json)?;
    let f = File::create(output)?;
    let mut bw = BufWriter::new(f);
    bw.write_all(json_pp.as_bytes())?;
    bw.flush()?;
    Ok(())
}

fn merge_json_files(files: &[String]) -> std::io::Result<serde_json::Value> {
    let mut f = File::open(files[0].clone())?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let mut result: serde_json::Value = serde_json::from_str(buf.as_str())?;
    let result_list = result["results"].as_array_mut().unwrap();
    for file in files.iter().skip(1) {
        let mut f = File::open(file)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        let mut val: serde_json::Value = serde_json::from_str(buf.as_str())?;
        let r = val["results"].as_array_mut().unwrap();
        result_list.append(r);
    }
    Ok(result)
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
