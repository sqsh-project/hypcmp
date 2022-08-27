#![allow(dead_code)]
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::Read};
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
    for (_, v) in c.run {
        println!("{:?}", v.to_hyperfine_params())
    }
}

#[derive(Deserialize, Debug)]
struct Benchmark {
    label: String,
    output: String,
    hyperfine_params: Vec<String>,
    run: HashMap<String, Run>,
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
