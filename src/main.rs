#![expect(unused)]

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::ValueEnum;

use crate::{
    args::parse_args,
    bwrap::{Bwrap, Mount},
    profiles::ProfileKind,
};

mod args;
mod bwrap;
mod path;
mod profiles;
mod sys;

fn main() -> Result<()> {
    let args = parse_args();
    println!("{args:#?}");

    let temp_dir =
        Utf8PathBuf::try_from(std::env::temp_dir()).expect("temp dir name should be in utf-8");

    let mut config = Bwrap::default();
    for p in ProfileKind::value_variants() {
        let profile = p.profile();
        if !profile.is_present() {
            continue;
        }
        for b in profile.bind {
            config.mounts.push(Mount::same(b, false));
        }
        for b in profile.ro_bind {
            config.mounts.push(Mount::same(b, true));
        }
        for (i, (target, content)) in profile.write.iter().enumerate() {
            let mut tmp = temp_dir.join(format!("{}_{i}", p.name()));
            if let Some(ext) = target.extension() {
                tmp.add_extension(ext);
            }
            std::fs::write(&tmp, content).with_context(|| "failed to write {tmp}")?;
            config.mounts.push(Mount::new(tmp, target, true));
        }
        config.allow_ignored.extend(profile.allow_ignored);
    }

    Ok(())
}
