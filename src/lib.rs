//! Command line tool for managing objects stored in a OneDrive service
use crate::auth::{get_auth_url, parse_token};
use simple_error::SimpleError;
use std::io::{stdin, stdout, Write};
use std::{error::Error, fmt::Debug};

type MyResult<T> = Result<T, Box<dyn Error>>;

use clap::{Parser, Subcommand};
pub mod auth;

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

/// Entry point function for the "init" subcommand
///
/// # Arguments
///
/// * `browser` - True if the user wants the browser to be automatically
///               launched by our app, and have the response from the
///               authentication request automatically intercepted
fn init_cmd(browser: bool) -> MyResult<()> {
    if browser {
        return Err(SimpleError::new("Feature not supported").into());
    }
    println!("Open this URL in your browser: {}", get_auth_url());

    let mut response_url = String::new();
    print!("Paste the response URL here: ");
    stdout().flush()?;
    stdin().read_line(&mut response_url)?;

    let token = parse_token(&response_url)?;

    // TODO: store token in a config file in the users home folder
    //       make it yaml format and set the mode to 600
    println!("Token is {}", token);
    Ok(())
}

/// Entrypoint function for our command line interface
pub fn run() -> MyResult<()> {
    let args = Args::parse();
    match args.cmd {
        SubCommand::Init { browser } => {
            println!("Interactive? {}", browser);
            init_cmd(browser)?;
        }
    }
    dbg!(args);
    Ok(())
}
