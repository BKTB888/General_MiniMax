#![feature(
    gca_const_items,
    gca_min_const_items,
    gca_macroless_args,
    generic_const_items,
    const_trait_impl,
    trait_alias
)]
#![allow(incomplete_features)]

pub mod coordinate;
pub mod game;
pub mod mixers;
pub mod player;
pub mod result;
pub mod state;
pub mod utils;

/// Widens a `u8` const generic to `usize` for use as an array length.
pub const AS_USIZE<const X: u8>: usize = X as usize;
