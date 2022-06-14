// Treat all enabled lints as errors
#![deny(clippy::all)]

fn main() {
    if let Err(e) = friendly_uploader::run() {
        eprintln!("{}", e);

        std::process::exit(1);
    }
}
