use log::{debug, error, trace};
use serde_json::Value;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::process::Command;
use tempfile::TempDir;

pub(crate) fn cleanup(tempfilelist: Vec<String>, dir: TempDir) -> std::io::Result<()> {
    for file in tempfilelist {
        debug!("Deleting file: {file:?}");
        drop(file)
    }
    debug!("Deleting folder: {dir:?}");
    dir.close()
}

pub(crate) fn is_git_dirty() -> std::io::Result<()> {
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

pub(crate) fn checkout(commit: String) -> std::io::Result<()> {
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

pub(crate) fn get_current_branch() -> std::io::Result<String> {
    let r = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?
        .stdout;
    Ok(to_string(r)) // return HEAD is detached
}

pub(crate) fn to_string(msg: Vec<u8>) -> String {
    let mut result = std::str::from_utf8(&msg).unwrap().to_string();
    trim_newline(&mut result);
    result
}

pub(crate) fn get_current_branch_or_id() -> std::io::Result<String> {
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

pub(crate) fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

pub(crate) fn get_current_commit() -> std::io::Result<String> {
    let r = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()?
        .stdout;
    Ok(to_string(r)) // return HEAD is detached
}

pub(crate) fn write_json_to_disk(json: Value) -> std::io::Result<()> {
    let json_pp = serde_json::to_string_pretty(&json)?;
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(json_pp.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

pub(crate) fn merge_json_files(files: &[String]) -> std::io::Result<serde_json::Value> {
    debug!("Merging files: {files:?}");
    let mut f = File::open(files[0].clone())?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    trace!("Read first file:\n{buf}");
    let result: serde_json::Value = serde_json::from_str(buf.as_str())?;
    let mut result = move_commit_label_to_cmd_name(result)?;
    let result_list = result["results"].as_array_mut().unwrap();
    for file in files.iter().skip(1) {
        debug!("Reading file: {file:?}");
        let mut f = File::open(file)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        trace!("Read file: {buf}");
        let val: serde_json::Value = serde_json::from_str(buf.as_str())?;
        let mut val = move_commit_label_to_cmd_name(val)?;
        let r = val["results"].as_array_mut().unwrap();
        result_list.append(r);
    }
    Ok(result)
}

pub(crate) fn move_commit_label_to_cmd_name(mut json: Value) -> std::io::Result<serde_json::Value> {
    let results = json["results"].as_array_mut().unwrap();
    for run in results {
        let commit = run["parameters"]
            .as_object()
            .and_then(|hm| hm.get("commit"));
        match commit {
            Some(suff) => {
                let old_name = run["command"].as_str().unwrap();
                let suffix = suff.as_str().unwrap();
                let new_name = [old_name, suffix].join("@");
                run["command"] = serde_json::Value::String(new_name.to_string());
            }
            None => (),
        }
    }
    Ok(json)
}

pub(crate) fn get_commit_ids() -> Option<Vec<String>> {
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

pub(crate) fn get_abbrev_commit_ids() -> Option<Vec<String>> {
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

pub(crate) fn get_branches() -> Option<Vec<String>> {
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

pub(crate) fn hyperfine_installed() -> std::io::Result<()> {
    let result = Command::new("which").arg("hyperfine").output()?;
    if !result.status.success() {
        let err = Error::new(ErrorKind::Other, "Hyperfine not installed");
        Err(err)
    } else {
        debug!("Hyperfine is installed");
        Ok(())
    }
}

pub(crate) fn get_tags() -> Option<Vec<String>> {
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
