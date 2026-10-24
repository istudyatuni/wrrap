use crate::args::*;

mod args;

fn main() -> Result<(), clap::Error> {
    let args = parse_args();
    println!("{args:#?}");

    Ok(())
}
