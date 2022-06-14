use std::{error::Error, fmt::Debug};

type MyResult<T> = Result<T, Box<dyn Error>>;

use clap::Parser;

/// Sample command line parser
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    // Character string input
    #[clap(short, long)]
    name: String,
}

pub fn run() -> MyResult<()> {
    let config = Args::parse();
    dbg!(config);
    Ok(())
}
