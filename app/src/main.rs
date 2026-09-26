#![feature(
    gca_const_items,
    gca_min_const_items,
    gca_macroless_args
)]
#![allow(incomplete_features)]

mod cli;

use clap::Parser;
use cli::CLI;

fn main() {
    CLI::parse().run();
}
