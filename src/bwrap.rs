use camino::{Utf8Path, Utf8PathBuf};

#[derive(Debug, Default)]
pub struct Bwrap {
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
