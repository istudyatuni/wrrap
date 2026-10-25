use anyhow::{Context, Result};

pub fn bytes2str(b: &[u8]) -> Result<String> {
    Ok(std::str::from_utf8(b)
        .context("expected utf-8")?
        .to_string())
}
