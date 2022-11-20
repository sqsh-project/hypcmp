use log::{debug, error, trace};
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Error, ErrorKind, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub fn cleanup(tempfilelist: Vec<String>, dir: TempDir) -> std::io::Result<()> {
    for file in tempfilelist {
        debug!("Deleting file: {file:?}");
        drop(file)
    }
    debug!("Deleting folder: {dir:?}");
    dir.close()
}

pub fn is_git_dirty() -> std::io::Result<()> {
    let st = Command::new("git").arg("diff").arg("--quiet").status()?;
    if st.success() {
        debug!("Git state is clean");
        Ok(())
    } else {
        error!("Git state is dirty");
        let err = Error::new(ErrorKind::Other, "Git is dirty");
        Err(err)
    }
}

pub fn checkout(commit: String) -> std::io::Result<()> {
    let id = get_current_branch_or_id()?;
    if id != commit {
        debug!("Git state changed!");
        let status = Command::new("git")
            .arg("checkout")
            .arg(commit.clone())
            .arg("--quiet")
            .status()?;
        if !status.success() {
            error!("Could not checkout from {} to {}", id, commit);
        }
    } else {
        debug!("Git state not changed");
    }
    Ok(()) // return HEAD is detached
}

pub fn get_current_branch() -> std::io::Result<String> {
    let r = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?
        .stdout;
    Ok(to_string(r)) // return HEAD is detached
}

pub fn to_string(msg: Vec<u8>) -> String {
    let mut result = std::str::from_utf8(&msg).unwrap().to_string();
    trim_newline(&mut result);
    result
}

pub fn get_current_branch_or_id() -> std::io::Result<String> {
    let mut br = get_current_branch()?;
    trim_newline(&mut br);
    if br == "HEAD" {
        debug!("Git not checked out at branch or tag");
        br = get_current_commit()?;
        debug!("Git at commit id: {br:?}");
        trim_newline(&mut br);
        Ok(br)
    } else {
        Ok(br)
    }
}

pub fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

pub fn get_current_commit() -> std::io::Result<String> {
    let r = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()?
        .stdout;
    Ok(to_string(r)) // return HEAD is detached
}

pub fn write_json_to_disk(json: Value, output: &String) -> std::io::Result<()> {
    let json_pp = serde_json::to_string_pretty(&json)?;
    let f = File::create(output)?;
    let mut bw = BufWriter::new(f);
    bw.write_all(json_pp.as_bytes())?;
    bw.flush()?;
    debug!("JSON file created: {output:?}");
    Ok(())
}

pub fn merge_json_files(files: &[String]) -> std::io::Result<serde_json::Value> {
    debug!("Merging files: {files:?}");
    let mut f = File::open(files[0].clone())?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    trace!("Read first file:\n{buf}");
    let mut result: serde_json::Value = serde_json::from_str(buf.as_str())?;
    let result_list = result["results"].as_array_mut().unwrap();
    for file in files.iter().skip(1) {
        debug!("Reading file: {file:?}");
        let mut f = File::open(file)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        trace!("Read file: {buf}");
        let mut val: serde_json::Value = serde_json::from_str(buf.as_str())?;
        let r = val["results"].as_array_mut().unwrap();
        result_list.append(r);
    }
    Ok(result)
}

pub fn get_commit_ids() -> Option<Vec<String>> {
    let result = Command::new("git")
        .arg("rev-list")
        .arg("--all")
        .output()
        .expect("Command failed");
    if result.status.success() {
        let s = to_string(result.stdout);
        let res: Vec<String> = s.split('\n').map(|s: &str| s.to_string()).collect();
        Some(res)
    } else {
        None
    }
}

pub fn get_abbrev_commit_ids() -> Option<Vec<String>> {
    let result = Command::new("git")
        .arg("rev-list")
        .arg("--all")
        .arg("--abbrev-commit")
        .output()
        .expect("Command failed");
    if result.status.success() {
        let s = to_string(result.stdout);
        let res: Vec<String> = s.split('\n').map(|s: &str| s.to_string()).collect();
        Some(res)
    } else {
        None
    }
}

pub fn get_branches() -> Option<Vec<String>> {
    let result = Command::new("git")
        .arg("branch")
        .arg("--all")
        .output()
        .expect("Command failed");
    if result.status.success() {
        let s = to_string(result.stdout);
        let res: Vec<String> = s.split('\n').map(|s: &str| s[2..].to_string()).collect();
        Some(res)
    } else {
        None
    }
}

pub fn hyperfine_installed() -> std::io::Result<()> {
    let result = Command::new("which").arg("hyperfine").output()?;
    if !result.status.success() {
        let err = Error::new(ErrorKind::Other, "Hyperfine not installed");
        Err(err)
    } else {
        debug!("Hyperfine is installed");
        Ok(())
    }
}

pub fn get_tags() -> Option<Vec<String>> {
    let result = Command::new("git")
        .arg("tag")
        .arg("--list")
        .output()
        .expect("Command failed");
    if result.status.success() {
        let out = to_string(result.stdout);
        let res: Vec<String> = out.split('\n').map(|s: &str| s.to_string()).collect();
        Some(res)
    } else {
        None
    }
}

#[cfg(feature = "plotly")]
use plotly::{
    box_plot::BoxPoints,
    common::Title,
    layout::{Axis, BoxMode, Layout},
    Plot,
};

#[cfg(feature = "plotly")]
pub fn export_to_html(json: &serde_json::Value, fname: PathBuf) -> std::io::Result<()> {
    let results = json["results"].as_array().unwrap();
    let mut plot = Plot::new();
    for run in results {
        let params = run["parameters"].as_object();
        let hover = match params {
            Some(p) => serde_json::to_string_pretty(&p)?,
            None => "".to_string(),
        };
        let basename = run["command"].as_str().unwrap();
        let times = run["times"].as_array().unwrap();
        let times: Vec<_> = times.iter().map(|v| v.as_f64().unwrap()).collect();
        let trace = plotly::BoxPlot::new(times)
            .name(&basename)
            .box_points(BoxPoints::All)
            .jitter(0.3)
            .hover_text(&hover);
        plot.add_trace(trace);
    }
    let layout = Layout::new()
        .y_axis(
            Axis::new()
                .title(Title::new("Benchmark Results"))
                .zero_line(true),
        )
        .box_mode(BoxMode::Group);
    plot.set_layout(layout);
    plot.to_html(fname);
    Ok(())
}

#[cfg(not(feature = "plotly"))]
pub fn export_to_html(_: &serde_json::Value, _: PathBuf) -> std::io::Result<()> {
    Ok(())
}
