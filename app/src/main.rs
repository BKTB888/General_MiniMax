#![feature(
    generic_const_args,
    min_generic_const_args,
    macroless_generic_const_args
)]
#![allow(incomplete_features)]

mod cli;

use clap::Parser;
use cli::CLI;

fn main() {
    CLI::parse().run();
}
