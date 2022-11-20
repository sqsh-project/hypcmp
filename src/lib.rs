use log::{debug, error, trace, warn};
use serde::Deserialize;
use std::{collections::HashMap, fmt::Display, fs::File, io::Read, path::PathBuf};
pub mod util;

#[derive(Deserialize, Debug)]
pub struct Benchmark {
    pub output: String,
    hyperfine_params: Vec<String>,
    pub run: HashMap<String, Run>,
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
    pub fn from_config(config: PathBuf) -> std::io::Result<Self> {
        debug!("Reading configuration file: {config:?}");
        let mut f = File::open(config)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        trace!("Content:\n {content}");
        let value = content.as_str();
        let result = toml::from_str(value)?;
        Ok(result)
    }
    pub fn to_hyperfine_params(&self) -> Vec<String> {
        self.hyperfine_params.clone()
    }
}

#[derive(Deserialize, Debug)]
pub struct Run {
    #[serde(default)]
    #[serde(deserialize_with = "from_commit")]
    commits: Option<Vec<String>>,
    cleanup: Option<String>,
    prepare: Option<String>,
    setup: Option<String>,
    shell: Option<String>,
    command: String,
}

fn from_commit<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<Vec<String>> = serde::Deserialize::deserialize(deserializer)?;

    match s {
        Some(vec) => {
            let (is_ok, failed) = check_validity_of_commit_ids(&vec);
            if is_ok {
                Ok(Some(vec))
            } else {
                let msg = format!("Following commits not found: {failed:?}");
                let err = serde::de::Error::custom(msg.replace('"', ""));
                error!("{err}");
                Err(err)
            }
        }
        None => Ok(None),
    }
}

fn check_validity_of_commit_ids(vec: &[String]) -> (bool, Vec<String>) {
    debug!("Commits: {vec:?}");
    let mut cs = util::get_branches().unwrap();
    let mut ct = util::get_tags().unwrap();
    let mut csa = util::get_abbrev_commit_ids().unwrap();
    let mut csb = util::get_commit_ids().unwrap();
    cs.append(&mut ct); // check first branches + tags
    cs.append(&mut csa); // then abbreviated commit ids
    cs.append(&mut csb); // then full commit ids
    debug!("Commits: {cs:?}");
    let mut not_found = Vec::new();
    let mut is_ok = true;
    for v in vec {
        if cs.iter().any(|s| s == v) {
            continue;
        } else {
            not_found.push(v.clone());
            is_ok = false;
        }
    }
    (is_ok, not_found)
}

impl Run {
    pub fn to_hyperfine_params(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        match (&self.shell, &self.commits, &self.setup) {
            (Some(_), Some(_), Some(_)) => {
                warn!("If 'commits' and 'setup' are set, the parameter shell cannot be set.");
            }
            (Some(sh), _, _) => {
                result.push("--shell".to_string());
                result.push(sh.clone());
            }
            _ => {}
        }
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
                let concat = format!("git checkout {{commit}} && {stp}", stp = scmd);
                result.push(concat);
            }
            (None, Some(_)) => {
                result.push("--setup".to_string());
                result.push("git checkout {commit}".to_string());
            }
            (Some(scmd), None) => {
                result.push("--setup".to_string());
                result.push(scmd.to_string());
            }
            (None, None) => (),
        }
        result.push(self.command.clone());
        result
    }
}
