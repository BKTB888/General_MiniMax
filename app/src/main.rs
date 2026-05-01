#![feature(generic_const_exprs)]
#![feature(decl_macro)]

use connect_k::state::ConnectKState;
use general_minimax::{
    game::Game,
    player::{
        evals::stupid_eval,
        players::randys_from_seed,
        search::{ABSearch, alphabeta},
    },
};

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
    let p1 = randys_from_seed(42);
    let p2 = alphabeta(stupid_eval).to_player(4);
    let num_games = 10_000;

    let mut game = Game::<Rules>::new(boxed![p1, p2]);
    game.print_stats(num_games, false);
    //game.print_play()
}
