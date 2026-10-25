use camino::Utf8PathBuf;
use clap::ValueEnum;

use crate::{path::path, sys};

#[derive(Debug, Default)]
pub struct Profile {
    /// Paths which used to detect whether profile should be activated
    pub search: Vec<Utf8PathBuf>,

    /// Detect profile by executable
    pub detect_exe: Vec<String>,

    /// Which paths to bind as read-write
    pub bind: Vec<Utf8PathBuf>,

    /// Which paths to bind as read-only
    pub ro_bind: Vec<Utf8PathBuf>,

    /// Create additional files. Will be bounded as read-only
    pub write: Vec<ProfileFile>,

    /// Allow paths to be mounted even if they're gitignored
    pub allow_ignored: Vec<Utf8PathBuf>,
}

#[derive(Debug)]
pub struct ProfileFile {
    pub path: Utf8PathBuf,
    pub content: String,
    pub executable: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ProfileKind {
    Base,
    Golang,
    NodejsNpm,
    NodejsYarn,
    Rust,
}

impl Profile {
    pub fn is_present(&self, exe: &str) -> bool {
        self.detect_exe.iter().any(|e| e == exe) || self.search.iter().all(|path| path.exists())
    }
}

impl ProfileFile {
    fn new(p: &str, content: &str) -> Self {
        Self {
            path: path(p),
            content: content.to_string(),
            executable: false,
        }
    }
    fn new_exec(p: &str, content: &str) -> Self {
        Self {
            path: path(p),
            content: content.to_string(),
            executable: true,
        }
    }
}

impl ProfileKind {
    pub fn profile(self) -> Profile {
        match self {
            Self::Base => base(),
            Self::Golang => golang(),
            Self::NodejsNpm => nodejs_npm(),
            Self::NodejsYarn => nodejs_yarn(),
            Self::Rust => rust(),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Golang => "golang",
            Self::NodejsNpm => "nodejs-npm",
            Self::NodejsYarn => "nodejs-yarn",
            Self::Rust => "rust",
        }
    }
}

pub fn base() -> Profile {
    let uid = sys::user_id();

    Profile {
        write: vec![
            // basic user info
            ProfileFile::new("/etc/passwd", &sys::get_passwd(&[uid, 65534])),
            ProfileFile::new("/etc/group", &sys::get_groups(&[uid, 65534])),
            // nix config
            ProfileFile::new("~/.config/nix/nix.conf", constants::NIX_CONF),
            // jj
            ProfileFile::new_exec("/usr/local/bin/jj", constants::JJ_WRAPPER),
        ],
        ..Default::default()
    }
}

pub fn golang() -> Profile {
    Profile {
        search: vec![path("go.mod")],
        ro_bind: vec![path("~/go")],
        ..Default::default()
    }
}

pub fn rust() -> Profile {
    // mounting separately because when mounting only ~/.cargo it's impossible
    // later to mount config.toml when it's a symlink
    let mut ro_bind: Vec<_> = ["bin", "build", "git", "registry"]
        .iter()
        .map(|dir| path(format!("~/.cargo/{dir}")))
        .collect();
    ro_bind.push(path("~/.rustup"));

    Profile {
        search: vec![path("Cargo.toml")],
        ro_bind,
        write: vec![ProfileFile::new(
            "~/.cargo/config.toml",
            constants::CARGO_CONFIG,
        )],
        allow_ignored: vec![path("./target")],
        ..Default::default()
    }
}

pub fn nodejs_yarn() -> Profile {
    Profile {
        search: vec![path("package.json"), path("yarn.lock")],
        ro_bind: vec![path("~/.config/yarn"), path("~/.yarn")],
        allow_ignored: vec![path("./node_modules")],
        ..Default::default()
    }
}

pub fn nodejs_npm() -> Profile {
    Profile {
        search: vec![path("package.json"), path("package-lock.json")],
        ro_bind: vec![path("~/.npm")],
        allow_ignored: vec![path("./node_modules")],
        ..Default::default()
    }
}

mod constants {
    // use mold instead of lld because otherwise build fails
    pub const CARGO_CONFIG: &str = r#"[target.x86_64-unknown-linux-gnu]
linker = "cc"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]"#;

    pub const NIX_CONF: &str = "experimental-features = nix-command flakes";

    pub const JJ_WRAPPER: &str = r#"#!/bin/env bash
exec /bin/jj --ignore-working-copy "$@""#;
}
