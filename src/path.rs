use std::sync::LazyLock;

use camino::Utf8PathBuf;

pub trait PathExt {
    /// Replace leading "~/" from path. If prefix not found, return unchanged
    fn expand_home(&self) -> Self;
    /// Canonicalize, but ignore if path is symlink
    fn as_absolute_no_symlink(&self) -> Option<Self>
    where
        Self: Sized;
}

impl PathExt for Utf8PathBuf {
    fn expand_home(&self) -> Self {
        if let Ok(path) = self.strip_prefix("~/")
            && let Ok(home) = home_dir()
        {
            return home.join(path);
        }
        self.to_owned()
    }
    fn as_absolute_no_symlink(&self) -> Option<Self> {
        if self.is_symlink() {
            return None;
        }
        self.canonicalize_utf8()
            .inspect_err(|e| eprintln!("failed to canonicalize path {self}: {e}"))
            .ok()
    }
}

pub fn path(path: impl AsRef<str>) -> Utf8PathBuf {
    Utf8PathBuf::from(path.as_ref()).expand_home()
}

fn home_dir() -> Result<Utf8PathBuf, camino::FromPathBufError> {
    static HOME: LazyLock<std::path::PathBuf> =
        LazyLock::new(|| dirs::home_dir().expect("HOME not defined"));

    (*HOME).clone().try_into()
}
