#![allow(dead_code)]
use std::{fs::File, io::Read, collections::HashMap};
use serde::Deserialize;

fn main() {
    let filename = "./sqsh-benchmark/examples/base.toml";
    let mut f = File::open(filename).unwrap();
    let mut content = String::new();
    f.read_to_string(&mut content).unwrap();
    let value = content.as_str();
    // let value = value.parse::<Value>().unwrap();
    let c: Benchmark = toml::from_str(value).unwrap();
    println!("{c:?}");
}

#[derive(Deserialize, Debug)]
struct Benchmark {
    name: String,
    output: String,
    hyperfine_params: Vec<String>,
    run: HashMap<String, ExeCommand>
}

trait Command {
    fn to_cmd(&self) -> String;
}

impl Command for Benchmark {
    fn to_cmd(&self) -> String {
        self.hyperfine_params.join(" ")
    }
}

#[derive(Deserialize, Debug)]
struct ExeCommand {
    command_params: Vec<String>,
    hashes: Option<Vec<String>>
}

impl Command for ExeCommand {
    fn to_cmd(&self) -> String {
        self.command_params.join(" ")
    }
}
