#![feature(generic_const_exprs)]
#![feature(decl_macro)]

use general_minimax::{
    game::Game,
    player::{
        evals::stupid_eval,
        players::randys_from_seed,
        search::{ABSearch, alphabeta},
    },
};
use mega_tictactoe::state::KInARowState;

macro boxed {
    [$x:expr] => {
        std::array::from_fn(|_| Box::new($x) as Box<dyn Player<_>>)
    },
    [$($x:expr),* $(,)?] => {
        [$(Box::new($x)),*]
    },
}

fn main() {
    type Rules = KInARowState<5>;
    let p1 = randys_from_seed(42);
    let p2 = alphabeta(stupid_eval).to_player(4);
    let num_games = 100_000;

    let mut game = Game::<Rules>::new(boxed![p1, p2]);
    game.print_stats(num_games, false);
    //game.print_play()
}
