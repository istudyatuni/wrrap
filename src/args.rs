use clap::{Parser, ValueEnum};

use crate::profiles::ProfileKind;

#[derive(Debug)]
pub struct Args {
    pub exe: String,
    pub profiles: Vec<ProfileKind>,
    pub exe_args: Vec<String>,
    pub bwrap_args: Vec<String>,
}

#[derive(Debug, Parser)]
#[clap(override_usage = "<EXE> [OPTIONS] --exe-args [EXE_ARGS] --bwrap-args [BWRAP_ARGS]")]
struct ArgsRaw {
    /// Executable to run
    exe: String,

    /// Extra profiles to enable
    #[arg(short, long)]
    profiles: Vec<Profile>,

    /// Arguments for executable
    // fake attribute for showing in help
    #[arg(long = "exe-args")]
    _exe_args: Vec<String>,

    /// Arguments for bwrap
    // fake attribute for showing in help
    #[arg(long = "bwrap-args")]
    _bwrap_args: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Profile {
    Golang,
    Rust,
}

impl From<Profile> for ProfileKind {
    fn from(value: Profile) -> Self {
        match value {
            Profile::Golang => Self::Golang,
            Profile::Rust => Self::Rust,
        }
    }
}

pub fn parse_args() -> Args {
    enum Group {
        ExeArgs,
        BwrapArgs,
    }

    let mut cli_args = vec![std::env::args().next().expect("app name should be known")];

    let mut exe = Vec::new();
    let mut bwrap = Vec::new();
    let mut current = None;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--exe-args" => current = Some(Group::ExeArgs),
            "--bwrap-args" => current = Some(Group::BwrapArgs),
            _ => match current {
                None => cli_args.push(arg),
                Some(Group::ExeArgs) => exe.push(arg),
                Some(Group::BwrapArgs) => bwrap.push(arg),
            },
        }
    }

    let args = ArgsRaw::parse_from(cli_args);
    Args {
        exe: args.exe,
        profiles: args.profiles.into_iter().map(Into::into).collect(),
        exe_args: exe,
        bwrap_args: bwrap,
    }
}
