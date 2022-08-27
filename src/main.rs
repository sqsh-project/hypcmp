#![allow(dead_code)]
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::Read, process::Command as PCommand};
mod cli;
use clap::Parser;

fn main() {
    let config = cli::Cli::parse();
    println!("{config:?}");
    let mut f = File::open(config.config).unwrap();
    let mut content = String::new();
    f.read_to_string(&mut content).unwrap();
    let value = content.as_str();
    // let value = value.parse::<Value>().unwrap();
    let c: Benchmark = toml::from_str(value).unwrap();
    println!("{c:?}");
    let b = c.run.get("past").unwrap().to_cmd_string();
    println!("{b:?}");
    let d = c.to_cmd_string();
    println!("{d:?}");
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
    command: String,
    commits: Option<Vec<String>>,
    cleanup: Option<String>,
    prepare: Option<String>,
    setup: Option<String>,
}
