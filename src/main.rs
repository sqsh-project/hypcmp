#![allow(dead_code)]
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::Read, process::Command as PCommand};

fn main() {
    let filename = "./sqsh-benchmark/examples/base.toml";
    let mut f = File::open(filename).unwrap();
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
    let f = PCommand::from(c.run.get("past").unwrap());
    println!("{f:?}");
}

#[derive(Deserialize, Debug)]
struct Benchmark {
    name: String,
    output: String,
    hyperfine_params: Vec<String>,
    run: HashMap<String, ExeCommand>,
}

trait Command {
    fn to_cmd_string(&self) -> String;
    fn to_cmd(&self) -> PCommand;
}

impl Command for Benchmark {
    fn to_cmd_string(&self) -> String {
        let cmd = self.to_cmd();
        format!("{:?}", cmd).replace('"', "")
    }
    fn to_cmd(&self) -> PCommand {
        let mut cmd = PCommand::new(self.hyperfine_params[0].clone());
        let vec: Vec<&str> = self
            .hyperfine_params
            .iter()
            .skip(1)
            .map(|x| x.as_str())
            .collect();
        cmd.args(vec);
        // cmd.args(["--export-os", self.output.as_str()]);
        cmd
    }
}

#[derive(Deserialize, Debug)]
struct ExeCommand {
    command_params: Vec<String>,
    hashes: Option<Vec<String>>,
}

impl Command for ExeCommand {
    fn to_cmd_string(&self) -> String {
        let cmd = self.to_cmd();
        format!("{:?}", cmd).replace('"', "")
    }
    fn to_cmd(&self) -> PCommand {
        let mut cmd = PCommand::new(self.command_params[0].clone());
        let vec: Vec<&str> = self
            .command_params
            .iter()
            .skip(1)
            .map(|x| x.as_str())
            .collect();
        cmd.args(vec);
        cmd
    }
}

impl From<&ExeCommand> for PCommand{
    fn from(cmd: &ExeCommand) -> Self {
        let mut result = PCommand::new(cmd.command_params[0].clone());
        let vec: Vec<&str> = cmd
            .command_params
            .iter()
            .skip(1)
            .map(|x| x.as_str())
            .collect();
        result.args(vec);
        result
    }
}