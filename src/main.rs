#![feature(generic_const_exprs)]
#![feature(trait_alias)]
#![feature(decl_macro)]

use crate::game::Game;
use crate::games::connect4::state::ConnectKState;
use crate::players::Evaluation;
use crate::players::Search;
use crate::players::{alphabeta, randys_from_seed, stupid_eval};

mod game;
mod games;
mod players;
mod result;
mod state;

macro boxed {
    [$x:expr] => {
        std::array::from_fn(|_| Box::new($x) as Box<dyn Player<_>>)
    },
    [$($x:expr),* $(,)?] => {
        [$(Box::new($x)),*]
    },
}

fn main() {
    type Rules = ConnectKState<6, 7>;
    let mut game = Game::<Rules>::new(boxed![
        randys_from_seed(42),
        alphabeta(stupid_eval).to_eval(4).to_player()
    ]);
    game.print_stats(10_000);

    /*
    game = Game::<Rules>::new(boxed![
        randys_from_seed(42),
        ai_from_eval(
            minimax_creator(
                4,
                mancala::eval
            )
        )
    ]);

    game.print_stats(10_000);

     */
}
