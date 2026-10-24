#![expect(unused)]

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::ValueEnum;

use crate::{
    args::parse_args,
    bwrap::{Bwrap, Mount},
    git::{GitInfo, GitStatus, git_status},
    profiles::ProfileKind,
};

mod args;
mod bwrap;
mod git;
mod path;
mod profiles;
mod string;
mod sys;

fn main() -> Result<()> {
    let args = parse_args();

    let temp_dir = Utf8PathBuf::try_from(std::env::temp_dir())
        .expect("temp dir name should be in utf-8")
        .join("tmp.wrrap");
    let empty_file = temp_dir.join("empty");
    let empty_dir = temp_dir.join("emptydir");
    std::fs::create_dir_all(&empty_dir).context("failed to create empty file")?;
    std::fs::write(&empty_file, "").context("failed to create empty dir")?;

    let cwd = Utf8PathBuf::try_from(std::env::current_dir().expect("current dir should be known"))
        .expect("current dir name should be in utf-8");

    let mut config = Bwrap {
        user: std::env::var("USER").context("failed to get USER var")?,
        ..Default::default()
    };

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

    let git = git_status(&cwd)?;

    let git_base = Utf8PathBuf::from(".git");
    if git_base.exists() && git_base.is_dir() {
        let path = git_base
            .canonicalize_utf8()
            .context("failed to canonicalize .git")?;
        config.mounts.push(Mount::same(path, true));
    }

    if let GitStatus::InRepo(GitInfo { ignored }) = &git {
        for path in ignored {
            if config.allow_ignored.contains(path) {
                continue;
            }
            if path.ends_with(".jj/") {
                config.mounts.push(Mount::same(path, true));
                continue;
            }

            let empty_source = if path.is_dir() {
                &empty_dir
            } else if !path.is_symlink() && path.is_file() {
                &empty_file
            } else {
                // todo: also mount symlink. currently not possible, see
                // https://github.com/containers/bubblewrap/issues/390
                continue;
            };
            config.mounts.push(Mount::new(empty_source, path, true));
        }
    }

    // todo: nix

    eprintln!("Running {} in isolation", args.exe);
    match &git {
        GitStatus::Outside => eprintln!("Running outside of git repo"),
        GitStatus::InIgnored => eprintln!("Current directory is git-ignored, mounting as-is"),
        GitStatus::InRepo(git_info) => (),
    }

    // eprintln!("bwrap config: {config:#?}");
    bwrap::run(config, args, &cwd)?;

    Ok(())
}
