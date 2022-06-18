// Treat all enabled lints as errors
#![deny(clippy::all)]

#[tokio::main]
async fn main() {
    if let Err(e) = friendly_uploader::run() {
        eprintln!("{}", e);

        std::process::exit(1);
    }
}
