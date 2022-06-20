//! Command line tool for managing objects stored in a OneDrive service
use crate::auth::{
    get_auth_data, get_auth_url, get_oauth_token_from_browser, parse_token, REDIRECT_URI,
};
use clap::{Parser, Subcommand};
use futures::executor;
use onedrive_api::{DriveLocation, ItemLocation, OneDrive};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::{error::Error, fmt::Debug};

pub mod auth;

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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    /// Primary authentication token used to connect to OneDrive
    /// If this token expires we need to use the refresh_token
    /// to renew it
    pub auth_token: String,
    /// Secondary authentication token used to renew the lifetime
    /// of the primary authentication token
    pub refresh_token: String,
}

/// Entry point function for the "init" subcommand
///
/// The command prompts the user for authentication parameters to OneDrive
/// and then generates a configuration file in the curent folder named
/// config.yml containing the authentication tokens retrieved from the
/// OAuth provider
///
/// # Arguments
///
/// * `browser` - True if the user wants the browser to be automatically
///               launched by our app, and have the response from the
///               authentication request automatically intercepted
fn init_cmd(browser: bool) -> MyResult<()> {
    let response_url = match browser {
        true => {
            println!("Waiting for OneDrive authentication request in your browser...");
            println!("Reference URL: {}", get_auth_url());
            println!("Listening for response on: {}", REDIRECT_URI);

            get_oauth_token_from_browser()?
        }
        false => {
            println!("Open this URL in your browser: {}", get_auth_url());
            print!("Paste the response URL here: ");
            stdout().flush()?;
            let mut temp = String::new();
            stdin().read_line(&mut temp)?;
            temp
        }
    };

    let token = parse_token(&response_url)?;
    let auth = get_auth_data(&token)?;

    let config = Configuration {
        auth_token: auth.access_token,
        refresh_token: auth.refresh_token,
    };

    let s = serde_yaml::to_string(&config)?;
    let mut file = File::create("config.yml")?;
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o600);
    file.set_permissions(perms)?;
    file.write_all(s.as_bytes())?;

    Ok(())
}

/// Entrypoint method for the 'ls' subcommand
/// Shows a directory listing of the root OneDrive folder
fn ls_cmd() -> MyResult<()> {
    let mut file = File::open("config.yml")?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let config: Configuration = serde_yaml::from_str(&s)?;

    let drive = OneDrive::new(config.auth_token, DriveLocation::me());
    let item = ItemLocation::root();
    let a = drive.list_children(item);
    let b = executor::block_on(a)?;
    for i in b {
        println!("{}", i.name.unwrap());
    }
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
    Ok(())
}
