//! Command line tool for managing objects stored in a OneDrive service
use clap::{Parser, Subcommand};
use commands::{init_cmd, ls_cmd, me_cmd, upload_cmd};
use futures::executor::block_on;
use std::{error::Error, fmt::Debug, path::PathBuf};
mod api;
mod auth;
mod commands;
mod configfile;

type MyResult<T> = Result<T, Box<dyn Error>>;

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
    /// List contents of root OneDrive folder
    Ls,
    /// Upload a new file to OneDrive root folder
    Upload {
        #[clap(short, long)]
        /// Path to the file to upload
        sourcefile: PathBuf,
    },
    /// Shows profile information for the currently logged in user
    Me,
}

/// Entrypoint function for our command line interface
pub fn run() -> MyResult<()> {
    let args = Args::parse();
    match args.cmd {
        SubCommand::Init { browser } => init_cmd(browser),
        SubCommand::Ls => block_on(ls_cmd()),
        SubCommand::Upload { sourcefile } => block_on(upload_cmd(&sourcefile)),
        SubCommand::Me => block_on(me_cmd()),
    }
}
