//! Command line tool for managing objects stored in a OneDrive service
use crate::auth::{get_auth_data, get_auth_url, parse_token};
use clap::{Parser, Subcommand};
use futures::executor;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use std::convert::Infallible;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::{error::Error, fmt::Debug};

use lazy_static::lazy_static;
use std::sync::Arc;
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

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if let Some(tx) = SHUTDOWN_TX.lock().await.take() {
        let _ = tx.send(());
    }
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
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        executor::block_on(SHUTDOWN_TX.lock()).replace(tx);

        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        let make_svc = make_service_fn(|_conn| async {
            // service_fn converts our function into a `Service`
            Ok::<_, Infallible>(service_fn(hello_world))
        });
        let server = Server::bind(&addr).serve(make_svc);
        let graceful = server.with_graceful_shutdown(async {
            rx.await.ok();
        });
        open::that(get_auth_url())?;
        executor::block_on(graceful)?;

        return Err(SimpleError::new("Feature not supported").into());
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
