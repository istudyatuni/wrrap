use std::sync::LazyLock;

use camino::Utf8PathBuf;

pub trait PathExt {
    /// Replace leading "~/" from path. If prefix not found, return unchanged
    fn expand_home(&self) -> Self;
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
}

pub fn path(path: impl AsRef<str>) -> Utf8PathBuf {
    Utf8PathBuf::from(path.as_ref()).expand_home()
}

fn home_dir() -> Result<Utf8PathBuf, camino::FromPathBufError> {
    static HOME: LazyLock<std::path::PathBuf> =
        LazyLock::new(|| dirs::home_dir().expect("HOME not defined"));

    (*HOME).clone().try_into()
}
