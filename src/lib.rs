use simple_error::SimpleError;
use std::{error::Error, fmt::Debug};

type MyResult<T> = Result<T, Box<dyn Error>>;

use clap::{Parser, Subcommand};

/// App for managing files on a OneDrive service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Initialize and authenticate the app
    Init {
        #[clap(short, long)]
        /// Can we intercept authentication requests from the browser?
        browser: bool,
    },
}

fn init(browser: bool) -> Result<(), SimpleError> {
    if browser {
        return Err(SimpleError::new("Feature not supported"));
    }
    Ok(())
}

pub fn run() -> MyResult<()> {
    let args = Args::parse();
    match args.cmd {
        SubCommand::Init { browser } => {
            println!("Interactive? {}", browser);
            init(browser)?;
        }
    }
    dbg!(args);
    Ok(())
}
