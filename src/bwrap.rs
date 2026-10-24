use std::process::Command;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

#[derive(Debug, Default)]
pub struct Bwrap {
    pub user: String,
    pub mounts: Vec<Mount>,
    pub envs: Vec<Env>,
    pub allow_ignored: Vec<Utf8PathBuf>,
}

#[derive(Debug)]
pub struct Mount {
    pub source: Utf8PathBuf,
    pub target: Utf8PathBuf,
    pub ro: bool,
}

#[derive(Debug)]
pub struct Env {
    pub name: String,
    pub value: String,
}

pub fn run(config: Bwrap, args: crate::args::Args, cwd: &Utf8Path) -> Result<()> {
    use args_builder::*;

    let mut cmd = Command::new("bwrap");
    cmd.arg("--die-with-parent")
        .args(pair("--dev", "/dev"))
        .args(pair("--proc", "/proc"))
        .args(pair("--tmpfs", "/tmp"))
        .args(ro_bind(&Utf8PathBuf::from("/nix")))
        .args(ro_bind_to("/run/current-system/sw/bin", "/bin"))
        .args(ro_bind_to(
            &format!("/etc/profiles/per-user/{}/bin", config.user),
            "/usr/bin",
        ))
        .arg("--unshare-all")
        .arg("--share-net")
        // network settings
        .args(ro_bind_real(&Utf8PathBuf::from("/etc/resolv.conf"))?)
        // ssl certs
        .args(ro_bind_real(&Utf8PathBuf::from(
            "/etc/ssl/certs/ca-bundle.crt",
        ))?)
        .args(ro_bind_real(&Utf8PathBuf::from(
            "/etc/ssl/certs/ca-certificates.crt",
        ))?)
        .arg("--clearenv")
        .args(env("PATH", &path_env(&["/bin", "/usr/bin"])))
        .args(inherit_env("HOME"))
        .args(inherit_env("TERM"))
        .args(inherit_env("EDITOR"))
        .args(inherit_env("LS_COLORS"))
        .args(inherit_env("NIX_PATH"))
        .args(bind(cwd));

    for mount in config.mounts {
        if mount.ro {
            cmd.args(ro_bind_to(mount.source.as_str(), mount.target.as_str()));
        } else {
            cmd.args(bind_to(mount.source.as_str(), mount.target.as_str()));
        }
    }

    // eprintln!("debug args: {:?}", cmd.get_args());

    cmd.arg(args.exe).status()?;

    Ok(())
}

impl Mount {
    pub fn new(source: impl AsRef<Utf8Path>, target: impl AsRef<Utf8Path>, ro: bool) -> Self {
        Self {
            source: source.as_ref().to_path_buf(),
            target: target.as_ref().to_path_buf(),
            ro,
        }
    }
    pub fn same(path: impl AsRef<Utf8Path>, ro: bool) -> Self {
        Self {
            source: path.as_ref().to_path_buf(),
            target: path.as_ref().to_path_buf(),
            ro,
        }
    }
}

impl Env {
    pub fn from_os(name: &str) -> Option<Self> {
        Some(Self {
            name: name.to_string(),
            value: std::env::var(name).ok()?,
        })
    }
}

mod args_builder {
    use std::ffi::OsString;

    use anyhow::{Context, Result};
    use camino::Utf8Path;

    use crate::string::os;

    pub fn pair<'a>(a1: &'a str, a2: &'a str) -> Vec<&'a str> {
        vec![a1, a2]
    }
    pub fn ro_bind_real(path: &Utf8Path) -> Result<Vec<String>> {
        let real = path
            .canonicalize_utf8()
            .with_context(|| format!("failed to get real path of {path}"))?;
        Ok(vec![
            "--ro-bind".to_string(),
            real.to_string(),
            path.to_string(),
        ])
    }
    pub fn ro_bind(path: &Utf8Path) -> Vec<&str> {
        vec!["--ro-bind", path.as_str(), path.as_str()]
    }
    pub fn bind(path: &Utf8Path) -> Vec<&str> {
        vec!["--bind", path.as_str(), path.as_str()]
    }
    pub fn ro_bind_to<'a>(from: &'a str, to: &'a str) -> Vec<&'a str> {
        vec!["--ro-bind", from, to]
    }
    pub fn bind_to<'a>(from: &'a str, to: &'a str) -> Vec<&'a str> {
        vec!["--bind", from, to]
    }

    pub fn env<'a>(name: &'a str, value: &'a str) -> Vec<&'a str> {
        vec!["--setenv", name, value]
    }
    pub fn inherit_env(name: &str) -> Vec<OsString> {
        vec![
            os("--setenv"),
            os(name),
            std::env::var_os(name).unwrap_or_default(),
        ]
    }
    pub fn path_env(values: &[&str]) -> String {
        values.join(":")
    }
}
