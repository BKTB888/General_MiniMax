#![feature(
    generic_const_args,
    min_generic_const_args,
    macroless_generic_const_args,
    generic_const_items
)]
#![allow(incomplete_features)]
#![feature(decl_macro)]

pub mod coordinate;
pub mod game;
pub mod mixers;
pub mod player;
pub mod result;
pub mod state;

pub use tt_call;

/// Widens a `u8` const generic to `usize` for use as an array length.
pub const AS_USIZE<const X: u8>: usize = X as usize;
