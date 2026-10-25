use camino::Utf8PathBuf;
use clap::{Parser, ValueEnum, error::ErrorKind};

use crate::profiles::ProfileKind;

#[derive(Debug)]
#[cfg_attr(test, derive(Default, PartialEq, Eq))]
pub struct Args {
    pub profiles: Vec<ProfileKind>,
    pub allow_ignored: Vec<Utf8PathBuf>,

    pub bwrap_args: Vec<String>,
    pub exe: String,
    pub exe_args: Vec<String>,
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

    /// Extra git-ignored paths to mount as-is instead of hiding
    #[arg(long, value_delimiter = ',')]
    allow_ignored: Vec<Utf8PathBuf>,

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
    NodejsNpm,
    NodejsYarn,
    Rust,
}

impl From<Profile> for ProfileKind {
    fn from(value: Profile) -> Self {
        match value {
            Profile::Golang => Self::Golang,
            Profile::NodejsNpm => Self::NodejsNpm,
            Profile::NodejsYarn => Self::NodejsYarn,
            Profile::Rust => Self::Rust,
        }
    }
}

fn fail(message: &str) -> ! {
    clap::Error::raw(ErrorKind::MissingRequiredArgument, message).exit()
}

pub fn parse_args() -> Args {
    parse_args_from(&std::env::args().collect::<Vec<_>>())
}

pub fn parse_args_from(args: &[String]) -> Args {
    let program = args
        .first()
        .cloned()
        .unwrap_or_else(|| env!("CARGO_BIN_NAME").into());
    let input = &args[1..];

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
        profiles: args.profiles.into_iter().map(Into::into).collect(),
        allow_ignored: args.allow_ignored,
        bwrap_args,
        exe,
        exe_args,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct Builder {
        args: Vec<String>,
    }

    impl Builder {
        fn arg(mut self, value: &str) -> Self {
            self.args(&[value])
        }
        fn bwrap(mut self) -> Self {
            self.arg("--bwrap-args")
        }
        fn sep(mut self) -> Self {
            self.arg("--")
        }
        fn args(mut self, value: &[&str]) -> Self {
            self.args.extend(value.iter().map(|s| s.to_string()));
            self
        }
    }

    #[test]
    fn test_parse_args() {
        const APP: &str = env!("CARGO_BIN_NAME");
        const EXE: &str = "/bin/echo";

        fn builder() -> Builder {
            Builder::default()
        }

        let table: Vec<(Builder, Args)> = vec![
            (
                builder().arg(EXE),
                Args {
                    exe: EXE.into(),
                    ..Default::default()
                },
            ),
            (
                builder().sep().arg(EXE),
                Args {
                    exe: EXE.into(),
                    ..Default::default()
                },
            ),
            (
                builder()
                    .args(&["--profiles", "rust"])
                    .sep()
                    .args(&[EXE, "hello", "world"]),
                Args {
                    exe: EXE.into(),
                    profiles: vec![ProfileKind::Rust],
                    exe_args: vec!["hello".into(), "world".into()],
                    ..Default::default()
                },
            ),
            (
                builder()
                    .bwrap()
                    .args(&["--ro-bind", "/usr", "/usr"])
                    .sep()
                    .args(&[EXE]),
                Args {
                    exe: EXE.into(),
                    exe_args: vec![],
                    bwrap_args: vec!["--ro-bind".into(), "/usr".into(), "/usr".into()],
                    ..Default::default()
                },
            ),
            (
                builder()
                    .args(&["--profiles", "rust"])
                    .bwrap()
                    .args(&["--unshare-net"])
                    .sep()
                    .args(&[EXE, "-n", "hello world"]),
                Args {
                    exe: EXE.into(),
                    profiles: vec![ProfileKind::Rust],
                    exe_args: vec!["-n".into(), "hello world".into()],
                    bwrap_args: vec!["--unshare-net".into()],
                    ..Default::default()
                },
            ),
            (
                builder().bwrap().sep().args(&[EXE]),
                Args {
                    exe: EXE.into(),
                    ..Default::default()
                },
            ),
        ];

        for (args, expected) in table {
            let args: Vec<_> = vec![APP.to_string()].into_iter().chain(args.args).collect();
            let actual = parse_args_from(&args);
            assert_eq!(actual, expected, "args: {args:?}");
        }
    }
}
