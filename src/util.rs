use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Error, ErrorKind, Read, Write};
use std::process::Command;
use tempfile::TempDir;

pub(crate) fn cleanup(tempfilelist: Vec<String>, dir: TempDir) -> std::io::Result<()> {
    for file in tempfilelist {
        drop(file)
    }
    dir.close()
}

pub(crate) fn is_git_dirty() -> std::io::Result<()> {
    let st = Command::new("git").arg("diff").arg("--quiet").status()?;
    if st.success() {
        Ok(())
    } else {
        let err = Error::new(ErrorKind::Other, "Git is dirty");
        Err(err)
    }
}

pub(crate) fn checkout(commit: String) -> std::io::Result<()> {
    let id = get_current_branch_or_id()?;
    if id != commit {
        Command::new("git").arg("checkout").arg(commit).arg("--quiet").status()?;
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
    Ok(std::str::from_utf8(&r).unwrap().to_string()) // return HEAD is detached
}

pub(crate) fn get_current_branch_or_id() -> std::io::Result<String> {
    let mut br = get_current_branch()?;
    trim_newline(&mut br);
    if br == "HEAD" {
        br = get_current_commit()?;
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
    Ok(std::str::from_utf8(&r).unwrap().to_string()) // return HEAD is detached
}

pub(crate) fn write_json_to_disk(json: Value, output: &String) -> std::io::Result<()> {
    let json_pp = serde_json::to_string_pretty(&json)?;
    let f = File::create(output)?;
    let mut bw = BufWriter::new(f);
    bw.write_all(json_pp.as_bytes())?;
    bw.flush()?;
    Ok(())
}

pub(crate) fn merge_json_files(files: &[String]) -> std::io::Result<serde_json::Value> {
    let mut f = File::open(files[0].clone())?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let mut result: serde_json::Value = serde_json::from_str(buf.as_str())?;
    let result_list = result["results"].as_array_mut().unwrap();
    for file in files.iter().skip(1) {
        let mut f = File::open(file)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        let mut val: serde_json::Value = serde_json::from_str(buf.as_str())?;
        let r = val["results"].as_array_mut().unwrap();
        result_list.append(r);
    }
    Ok(result)
}
