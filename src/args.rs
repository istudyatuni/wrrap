use clap::Parser;

#[derive(Debug)]
#[expect(unused)]
pub struct Args {
    arg: u32,
    exe: Vec<String>,
    bwrap: Vec<String>,
}

#[derive(Debug, Parser)]
struct ArgsRaw {
    #[arg(short, long)]
    arg: u32,

    /// Arguments for executable
    // fake attribute for showing in help
    #[arg(long = "exe-args")]
    _exe: Vec<String>,

    /// Arguments for bwrap
    // fake attribute for showing in help
    #[arg(long = "bwrap-args")]
    _bwrap: Vec<String>,
}

pub fn parse_args() -> Args {
    enum Group {
        Exe,
        Bwrap,
    }

    let mut cli_args = vec![std::env::args().next().expect("app name should be known")];

    let mut exe = Vec::new();
    let mut bwrap = Vec::new();
    let mut current = None;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--exe-args" => current = Some(Group::Exe),
            "--bwrap-args" => current = Some(Group::Bwrap),
            _ => match current {
                None => cli_args.push(arg),
                Some(Group::Exe) => exe.push(arg),
                Some(Group::Bwrap) => bwrap.push(arg),
            },
        }
    }

    let args = ArgsRaw::parse_from(cli_args);
    Args {
        arg: args.arg,
        exe,
        bwrap,
    }
}
