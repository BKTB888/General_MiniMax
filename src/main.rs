#![feature(generic_const_exprs)]
#![feature(trait_alias)]
#![feature(decl_macro)]

use crate::players::{randy, Player, human, minimax_creator, stupid_eval, ai_from_eval, alphabeta_creator};
use crate::game::Game;
use crate::games::connect4::state::ConnectKState;
use crate::games::mancala::state::MancalaState;
use crate::games::mega_tictactoe::player::human_kinrow;
use crate::games::mega_tictactoe::state::KInARowState;

mod state;
mod games;
mod result;
mod game;
mod players;

macro boxed {
    [$x:expr] => {
        std::array::from_fn(|_| Box::new($x) as Box<dyn Player<_>>)
    },
    [$($x:expr),* $(,)?] => {
        [$(Box::new($x)),*]
    },
}

fn main() {
    type Rules = ConnectKState<4, 6, 7, 2>;
    let game = Game::<Rules>::new(boxed![
        randy, randy
    ]);
    game.print_stats(1_000_000);
}
