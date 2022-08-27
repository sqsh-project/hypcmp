#![allow(dead_code)]
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::Read, process::Command};
mod cli;
use clap::Parser;

fn main() {
    let config = cli::Cli::parse();
    println!("{config:?}");

    let mut f = File::open(config.config).unwrap();
    let mut content = String::new();
    f.read_to_string(&mut content).unwrap();
    let value = content.as_str();

    let c: Benchmark = toml::from_str(value).unwrap();
    println!("Common Settings:");
    println!("{:?}", c.to_hyperfine_params());
    for (k, v) in c.run.iter() {
        println!("Subcommand Settings: {k:?}");
        println!("{:?}", v.to_hyperfine_params())
    }
    let mut tmp_outputs: Vec<String> = Vec::new();
    for (k, v) in c.run.iter() {
        let mut cmd = Command::new("hyperfine");
        cmd.args(c.to_hyperfine_params());

        let mut json = vec!["--export-json".to_string()];
        let output = format!("/tmp/{k:?}.json").replace('"', "");
        tmp_outputs.push(output.clone());
        json.push(output);
        cmd.args(json);

        cmd.args(v.to_hyperfine_params());
        println!("CMD '{k:?}': {cmd:?}");
        cmd.status().expect("Failed");  // Execute command back to back
    }
    merge_json_files(&tmp_outputs);
}

fn merge_json_files(files: &Vec<String>) {
    let mut f = File::open(files[0].clone()).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    let mut result: serde_json::Value = serde_json::from_str(buf.as_str()).unwrap();
    let result_list = result["results"].as_array_mut().unwrap();
    for file in files.iter().skip(1) {
        let mut f = File::open(file).unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(buf.as_str()).unwrap();
        // println!("{:?} {:?}", v["results"][0]["command"], v["results"][0]["median"]);
        result_list.push(v);
    }
    println!("{:?}", serde_json::to_string_pretty(&result));
}

#[derive(Deserialize, Debug)]
struct Benchmark {
    label: String,
    output: String,
    hyperfine_params: Vec<String>,
    run: Box<HashMap<String, Run>>,
}

impl Benchmark {
    fn to_hyperfine_params(&self) -> Vec<String> {
        let result = self.hyperfine_params.clone();
        // result.push("--export-json".to_string());
        // result.push(self.output.clone());
        result
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
