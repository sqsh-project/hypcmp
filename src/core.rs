use serde::Deserialize;
use std::{collections::HashMap, fmt::Display, fs::File, io::Read, path::PathBuf};

#[derive(Deserialize, Debug)]
pub(crate) struct Benchmark {
    pub(crate) output: String,
    hyperfine_params: Vec<String>,
    pub(crate) run: HashMap<String, Run>,
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
    pub(crate) fn from_config(config: PathBuf) -> std::io::Result<Self> {
        let mut f = File::open(config)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        let value = content.as_str();
        let result = toml::from_str(value)?;
        Ok(result)
    }
    pub(crate) fn to_hyperfine_params(&self) -> Vec<String> {
        self.hyperfine_params.clone()
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct Run {
    commits: Option<Vec<String>>,
    cleanup: Option<String>,
    prepare: Option<String>,
    setup: Option<String>,
    command: String,
}

impl Run {
    pub(crate) fn to_hyperfine_params(&self) -> Vec<String> {
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
