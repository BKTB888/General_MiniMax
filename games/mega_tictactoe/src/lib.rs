#![feature(
    gca_const_items,
    gca_min_const_items,
    gca_macroless_args,
    generic_const_items,
    const_trait_impl
)]
#![allow(incomplete_features)]

mod evaluation;
pub(crate) mod map;
pub mod player;
pub mod state;

pub use evaluation::eval;
