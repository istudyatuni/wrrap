use std::ffi::OsString;

use anyhow::{Context, Result};

pub fn os(s: &str) -> OsString {
    OsString::from(s)
}

pub fn bytes2str(b: &[u8]) -> Result<String> {
    Ok(std::str::from_utf8(b)
        .context("expected utf-8")?
        .to_string())
}
