mod args;
mod cli;
mod common;
mod domain;
mod errors;
mod handle;
mod persistence;
mod tui;
mod utils;

use args::Args;
use clap::Parser;
use handle::handle;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let result = handle(args).await;

    if let Err(error) = &result {
        eprintln!("Error: {}", error);
        if let Some(c) = error.code() {
            eprintln!(
                "
------

This error is unexpected.
Let @dhth know about this via https://github.com/dhth/bmm/issues (mention the error code E{}).",
                c
            );
        }

        if let Some(follow_up) = error.follow_up() {
            eprintln!(
                "
{}",
                follow_up
            );
        }
        std::process::exit(1);
    }
}
