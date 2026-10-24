use std::ffi::OsString;

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use same_file::is_same_file;

use crate::{git::cmd::git_output, string::os};

#[derive(Debug)]
pub enum GitStatus {
    /// Outside of git
    Outside,
    /// Inside of gitignored directory
    InIgnored,
    /// In repository root
    InRepo(GitInfo),
}

#[derive(Debug)]
pub struct GitInfo {
    pub ignored: Vec<Utf8PathBuf>,
}

pub fn git_status() -> Result<GitStatus> {
    if !cmd::git_status(&["rev-parse"]) {
        return Ok(GitStatus::Outside);
    }
    let cwd = Utf8PathBuf::try_from(std::env::current_dir().expect("current dir should be known"))
        .expect("current dir name should be in utf-8");
    if cmd::git_status(&["check-ignore", "--quiet", cwd.as_str()]) {
        return Ok(GitStatus::InIgnored);
    }

    let git_root = git_output(&["rev-parse", "--show-toplevel"])
        .context("failed to get git top-level")?
        .trim()
        .to_string();
    let ignored = git_output(&["status", "--ignored", "--porcelain=v1"])
        .context("failed to list git ignored paths")?
        .lines()
        .filter(|line| line.starts_with("!! "))
        .map(|line| Utf8PathBuf::from(line.trim_start_matches("!! ")))
        .collect();
    if is_same_file(&git_root, &cwd)
        .with_context(|| format!("git root {git_root} and cwd {cwd} should be accessible"))?
    {
        return Ok(GitStatus::InRepo(GitInfo { ignored }));
    }

    let subdir = cwd
        .strip_prefix(git_root)
        .expect("expected current dir be inside git root");
    Ok(GitStatus::InRepo(GitInfo {
        ignored: ignored
            .into_iter()
            .filter_map(|path| Some(path.strip_prefix(subdir).ok()?.to_path_buf()))
            .collect(),
    }))
}

pub mod cmd {
    use std::{
        ffi::OsStr,
        process::{Command, Output},
    };

    use anyhow::{Context, Result, bail};

    use crate::string::bytes2str;

    pub fn git_status(args: &[impl AsRef<OsStr>]) -> bool {
        git(args).is_ok_and(|o| o.status.success())
    }
    pub fn git_output(args: &[impl AsRef<OsStr>]) -> Result<String> {
        Ok(bytes2str(&git(args)?.stdout)
            .context("parsing git output")?
            .to_string())
    }

    fn git(args: &[impl AsRef<OsStr>]) -> Result<Output> {
        let output = Command::new("git")
            .args(args)
            .output()
            .context("failed to run git")?;
        if !output.status.success() {
            bail!(
                "git command with args {:?} failed, output:\n{}\n{}",
                args.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
                bytes2str(&output.stdout).unwrap(),
                bytes2str(&output.stderr).unwrap()
            );
        }
        Ok(output)
    }
}
