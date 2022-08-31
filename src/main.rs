use clap::Parser;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{BufWriter, Error, ErrorKind, Read, Write},
    path::PathBuf,
    process::Command,
};
use tempfile::TempDir;
mod cli;

fn main() -> std::io::Result<()> {
    let config = cli::Cli::parse();
    // println!("{config:?}");

    is_git_dirty()?;

    let c = Benchmark::from_config(config.config)?;
    // println!("{c}");

    let dir = tempfile::tempdir()?;
    let mut files_to_be_merged: Vec<String> = Vec::new();
    let current_branch = get_current_branch_or_id()?;
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
    let json = merge_json_files(&files_to_be_merged)?;
    write_json_to_disk(json, &c.output)?;
    cleanup(files_to_be_merged, dir)?;
    checkout(current_branch)?;
    Ok(())
}

fn cleanup(tempfilelist: Vec<String>, dir: TempDir) -> std::io::Result<()> {
    for file in tempfilelist {
        drop(file)
    }
    dir.close()
}

fn is_git_dirty() -> std::io::Result<()> {
    let st = Command::new("git").arg("diff").arg("--quiet").status()?;
    if st.success() {
        Ok(())
    } else {
        let err = Error::new(ErrorKind::Other, "Git is dirty");
        Err(err)
    }
}

fn checkout(commit: String) -> std::io::Result<()> {
    let id = get_current_branch_or_id()?;
    if id != commit {
        Command::new("git").arg("checkout").arg(commit).status()?;
    }
    Ok(()) // return HEAD is detached
}

fn get_current_branch() -> std::io::Result<String> {
    let r = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?
        .stdout;
    Ok(std::str::from_utf8(&r).unwrap().to_string()) // return HEAD is detached
}

fn get_current_branch_or_id() -> std::io::Result<String> {
    let mut br = get_current_branch()?;
    trim_newline(&mut br);
    if br == "HEAD" {
        br = get_current_commit()?;
        trim_newline(&mut br);
        Ok(br)
    } else {
        Ok(br)
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

fn get_current_commit() -> std::io::Result<String> {
    let r = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()?
        .stdout;
    Ok(std::str::from_utf8(&r).unwrap().to_string()) // return HEAD is detached
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
