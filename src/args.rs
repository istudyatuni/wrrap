use clap::{Parser, ValueEnum, error::ErrorKind};

use crate::profiles::ProfileKind;

#[derive(Debug)]
pub struct Args {
    pub exe: String,
    pub profiles: Vec<ProfileKind>,
    pub exe_args: Vec<String>,
    pub bwrap_args: Vec<String>,
}

#[derive(Debug, Parser)]
#[clap(override_usage = "
  wrrap <EXE>
  wrrap [OPTIONS] -- <EXE> [EXE_ARGS]...
  wrrap [OPTIONS] --bwrap-args [BWRAP_ARGS]... -- <EXE> [EXE_ARGS]...")]
struct ArgsRaw {
    /// Extra profiles to enable
    #[arg(short, long)]
    profiles: Vec<Profile>,

    /// Arguments for bwrap (before `--`)
    // fake attribute for showing in help
    #[arg(
        long = "bwrap-args",
        value_name = "BWRAP_ARGS",
        num_args = 0..,
        allow_hyphen_values = true,
    )]
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

fn fail(message: &str) -> ! {
    clap::Error::raw(ErrorKind::MissingRequiredArgument, message).exit()
}

pub fn parse_args() -> Args {
    let argv: Vec<String> = std::env::args().collect();
    let program = argv
        .first()
        .cloned()
        .unwrap_or_else(|| env!("CARGO_BIN_NAME").into());
    let input = &argv[1..];

    let separator = input.iter().position(|arg| arg == "--");

    let (options, bwrap_args, command) = match separator {
        Some(sep) => {
            let before_separator = &input[..sep];

            if let Some(marker) = before_separator
                .iter()
                .position(|arg| arg == "--bwrap-args")
            {
                (
                    before_separator[..marker].to_vec(),
                    before_separator[marker + 1..].to_vec(),
                    input[sep + 1..].to_vec(),
                )
            } else {
                (
                    before_separator.to_vec(),
                    Vec::new(),
                    input[sep + 1..].to_vec(),
                )
            }
        }
        None => {
            if input.iter().any(|arg| arg == "--bwrap-args") {
                fail("`--bwrap-args` must be followed by `--`");
            }

            // without "--", accept a bare executable and its arguments
            if input.first().is_some_and(|arg| !arg.starts_with('-')) {
                (Vec::new(), Vec::new(), input.to_vec())
            } else {
                (input.to_vec(), Vec::new(), Vec::new())
            }
        }
    };

    let args = ArgsRaw::parse_from(std::iter::once(program).chain(options));

    let (exe, exe_args) = match command.split_first() {
        Some((exe, args)) => (exe.clone(), args.to_vec()),
        None => fail("missing executable, expected `<EXE>`"),
    };

    Args {
        exe,
        profiles: args.profiles.into_iter().map(Into::into).collect(),
        exe_args,
        bwrap_args,
    }
}
