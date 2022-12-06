//! # Benchmark and Runs
//! This module defines the two main components of the application: Benchmark and Runs.
//!
//! A Benchmark defines the complete hyperfine setup. It consists of individual
//! and shared configurations between Runs. A Run is defined as a single timed
//! command by `hyperfine`.
use crate::util;
use log::{debug, error, trace, warn};
use serde::Deserialize;
use std::{collections::HashMap, fmt::Display, fs::File, io::Read, path::PathBuf};

/// Transformation to hyperfine parameters
pub trait Hyperfined {
    fn to_hyperfine(&self) -> Vec<String>;
    fn to_hyperfine_with_json(&self, json: &str) -> Vec<String> {
        let mut params = self.to_hyperfine();
        params.push("--export-json".to_string());
        params.push(json.to_string());
        params
    }
    fn to_hyperfine_with_json_and_annotations(&self, json: &str, kw: &HashMap<String, String>) -> Vec<String> {
        let mut params = self.to_hyperfine_with_json(json);
        let mut annotations: Vec<_> = kw.iter().map(|(k, v)| util::generate_jq_cmd(&k, &v, json)).collect();
        match &params.iter().position(|x| x == &"--cleanup".to_string()) {
            Some(ix) => {
                // Cleanup used in command and annotations must be prepand
                annotations.push(params.get(ix+1).unwrap().clone());
                params.insert(ix + 1, annotations.join("&&"));
            },
            None => {
                // Cleanup not used, annotations can be added as cleanup
                params.push("--cleanup".to_string());
                params.push(annotations.join("&&"));
            }
        }
        params
    }
}

/// Configuration for a complete Benchmark set consisting of several Runs
#[derive(Deserialize, Debug)]
pub(crate) struct Benchmark {
    hyperfine_params: Vec<String>,
    pub(crate) run: HashMap<String, Run>,
}

impl Display for Benchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Common Settings:")?;
        writeln!(f, "{:?}", self.to_hyperfine())?;
        for (k, v) in self.run.iter() {
            writeln!(f, "Subcommand Settings: {k:?}")?;
            writeln!(f, "{:?}", v.to_hyperfine())?;
        }
        writeln!(f)
    }
}

impl Benchmark {
    /// Setup Benchmark from a toml configuration file
    pub(crate) fn from_config(config: PathBuf) -> std::io::Result<Self> {
        debug!("Reading configuration file: {config:?}");
        let mut f = File::open(config)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        trace!("Content:\n {content}");
        let value = content.as_str();
        let result = toml::from_str(value)?;
        Ok(result)
    }
}

impl Hyperfined for Benchmark {
    /// Return common configuration of the Benchmark
    fn to_hyperfine(&self) -> Vec<String> {
        self.hyperfine_params.clone()
    }
}

/// A timed run executed by `hyperfine`
#[derive(Deserialize, Debug)]
pub(crate) struct Run {
    #[serde(default)]
    #[serde(deserialize_with = "from_commit")]
    commits: Option<Vec<String>>,
    cleanup: Option<String>,
    prepare: Option<String>,
    setup: Option<String>,
    shell: Option<String>,
    annotations: Option<HashMap<String, String>>,
    command: String,
}

/// Custom deserialization function for commits checking for correctness of commit-ids
fn from_commit<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<Vec<String>> = serde::Deserialize::deserialize(deserializer)?;

    match s {
        Some(vec) => match check_correctness_of_commit_ids(&vec) {
            Commits::Valid => Ok(Some(vec)),
            Commits::SpecialCaseAll(vec) => {
                debug!("Running on {vec:?}");
                Ok(Some(vec))
            }
            Commits::SomeInvalid(vec) => {
                let msg = format!("Following commits not found: {vec:?}");
                let err = serde::de::Error::custom(msg.replace('"', ""));
                error!("{err}");
                Err(err)
            }
            Commits::NoTagsFound => {
                let msg = format!("No tags found in repository");
                let err = serde::de::Error::custom(msg.replace('"', ""));
                error!("{err}");
                Err(err)
            }
        },
        None => Ok(None),
    }
}

/// Possible states after checking of entered commits
enum Commits {
    Valid,
    NoTagsFound,
    SpecialCaseAll(Vec<String>),
    SomeInvalid(Vec<String>),
}

/// Checking correctness of commit ids
fn check_correctness_of_commit_ids(vec: &[String]) -> Commits {
    debug!("Commits: {vec:?}");
    if vec.iter().any(|s| s == "--all") {
        // Benchmark should run on all commits
        return Commits::SpecialCaseAll(util::get_abbrev_commit_ids().unwrap());
    } else if vec.iter().any(|s| s == "--branches") {
        // Benchmark should run on all branches
        return Commits::SpecialCaseAll(util::get_branches().unwrap());
    } else if vec.iter().any(|s| s == "--tags") {
        // Benchmark should run on all tagged commits
        let vec = util::get_tags().unwrap();
        if vec[0] == "" {
            return Commits::NoTagsFound;
        } else {
            return Commits::SpecialCaseAll(vec);
        }
    } else if vec.iter().any(|s| s.starts_with("--since"))
        || vec.iter().any(|s| s.starts_with("--before"))
    {
        // Benchmark should run on all commits within two commits (incl.)
        let since = vec
            .iter()
            .find(|&f| f.starts_with("--since"))
            .and_then(|s| s.strip_prefix("--since="));
        let before = vec
            .iter()
            .find(|&f| f.starts_with("--before"))
            .and_then(|s| s.strip_prefix("--before="));
        return Commits::SpecialCaseAll(util::get_commit_ids_since_before(since, before).unwrap());
    } else {
        // Benchmark should run only on given commits (preparation)
        let mut commits = Vec::new();
        commits.append(&mut util::get_branches().unwrap());
        commits.append(&mut util::get_tags().unwrap());
        commits.append(&mut util::get_abbrev_commit_ids().unwrap());
        commits.append(&mut util::get_commit_ids().unwrap());
        let not_found: Vec<_> = vec
            .iter()
            .filter(|&c| !commits.iter().any(|s| s == c))
            .map(|s| s.clone())
            .collect();
        if not_found.is_empty() {
            // Benchmark should run on all commits (all were valid)
            Commits::Valid
        } else {
            // Benchmark should run on all commits (some commits not found)
            Commits::SomeInvalid(not_found)
        }
    }
}

impl Hyperfined for Run {
    /// Return custom-part of `hyperfine` configuration of Run
    fn to_hyperfine(&self) -> Vec<String> {
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
