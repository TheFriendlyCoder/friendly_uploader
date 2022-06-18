//! Command line tool for managing objects stored in a OneDrive service
use crate::auth::{get_auth_url, parse_token};
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
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
    /// Show folder listing
    Ls,
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub auth_token: String,
    pub refresh_token: String,
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

    let config = Configuration {
        auth_token: String::from(token),
        refresh_token: String::from(""),
    };
    let s = serde_yaml::to_string(&config)?;
    let mut file = File::create("config.yml")?;
    file.write_all(s.as_bytes())?;

    Ok(())
}

fn ls_cmd() -> MyResult<()> {
    let mut file = File::open("config.yml")?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let config: Configuration = serde_yaml::from_str(&s)?;
    println!("Token is {}", config.auth_token);
    Ok(())
}
/// Entrypoint function for our command line interface
pub fn run() -> MyResult<()> {
    let args = Args::parse();
    match args.cmd {
        SubCommand::Init { browser } => {
            init_cmd(browser)?;
        }
        SubCommand::Ls => {
            ls_cmd()?;
        }
    }
    dbg!(args);
    Ok(())
}
