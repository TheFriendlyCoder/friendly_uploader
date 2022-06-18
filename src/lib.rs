//! Command line tool for managing objects stored in a OneDrive service
use crate::auth::{get_auth_data, get_auth_url, parse_token};
use clap::{Parser, Subcommand};
use futures::executor;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use std::convert::Infallible;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::{error::Error, fmt::Debug};
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;

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

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    /// Primary authentication token used to connect to OneDrive
    /// If this token expires we need to use the refresh_token
    /// to renew it
    pub auth_token: String,
    /// Secondary authentication token used to renew the lifetime
    /// of the primary authentication token
    pub refresh_token: String,
}

lazy_static! {
    /// Channel used to send shutdown signal - wrapped in an Option to allow
    /// it to be taken by value (since oneshot channels consume themselves on
    /// send) and an Arc<Mutex> to allow it to be safely shared between threads
    static ref SHUTDOWN_TX: Arc<Mutex<Option<Sender<()>>>> = <_>::default();
}

async fn onedrive_oauth_response(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // TODO: store client code in a mutex so it can be passed back to the caller
    // TODO: implement error handling
    // TODO: move this auth code into auth module
    // TODO: make listening port use configurable via command line options
    // TODO: have the browser window open a pop-up dialog telling the user to
    //       go back to their terminal window, then when the dialog is closed
    //       close the browser window/tab
    // TODO: consider setting a timeout on the dynamic loading process so the
    //       tool doesn't block indefinitely
    // TODO: put a status message on the console before launching the browser
    //       window so they aren't left wondering what is going on
    // https://users.rust-lang.org/t/how-to-stop-hypers-server/26322
    // https://stackoverflow.com/questions/63599177/how-do-i-terminate-a-hyper-server-after-fulfilling-one-request
    // https://github.com/IntrepidPig/orca/blob/cf20d349d8cea92eab66aeb84838541e4fda29e4/src/net/auth.rs
    let params = req.uri().to_string();
    let url = format!("{}{}", "http://127.0.0.1:8080", params);
    let token = parse_token(&url).unwrap();
    let msg = format!("URL {} with token {}", url, token);
    Ok(Response::new(msg.into()))
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
        let (sender, receiver) = tokio::sync::oneshot::channel::<()>();
        executor::block_on(SHUTDOWN_TX.lock()).replace(sender);

        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        let make_svc = make_service_fn(|_conn| async {
            // service_fn converts our function into a `Service`
            let res = service_fn(onedrive_oauth_response);
            SHUTDOWN_TX.lock().await.take().unwrap().send(()).ok();
            Ok::<_, Infallible>(res)
        });
        let server = Server::bind(&addr).serve(make_svc);
        let graceful = server.with_graceful_shutdown(async { receiver.await.unwrap() });

        open::that(get_auth_url())?;
        executor::block_on(graceful)?;

        return Ok(());
    }
    println!("Open this URL in your browser: {}", get_auth_url());

    let mut response_url = String::new();
    print!("Paste the response URL here: ");
    stdout().flush()?;
    stdin().read_line(&mut response_url)?;

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
