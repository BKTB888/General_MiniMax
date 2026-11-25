#![feature(
    gca_const_items,
    gca_min_const_items,
    gca_macroless_args
)]
#![expect(incomplete_features)]

mod cli;

use clap::Parser;
use cli::Cli;

fn main() {
    Cli::parse().run();
}
