use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use rayon::{iter::ParallelIterator, prelude::IntoParallelRefIterator};

use crate::{
    player::{
        evals::Evaluation,
        players::Player,
        search::EvalResult::{Loss, Win},
        transposition_table::{TTBound, TTable},
    },
    state::GameState,
};

pub trait Search<S: GameState>: Fn(&mut S, u8) -> EvalResult + Sync + Sized {
    fn to_eval(self, depth: u8) -> impl Evaluation<S>
    where
        Self: Send,
    {
        move |state| self(&mut state.clone(), depth)
    }
    fn to_player(self, depth: u8) -> impl Player<S> {
        move |state| {
            state
                .candidate_moves()
                .par_iter()
                .copied()
                .map_init(
                    || state.clone(), // one clone per worker
                    |state, game_move| {
                        state.make_move(game_move);
                        let result = self(state, depth);
                        state.undo();
                        (game_move, result)
                    },
                )
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap()
                .0
        }
    }
}
impl<S: GameState, F: Fn(&mut S, u8) -> EvalResult + Sync> Search<S> for F {}

/// Takes a state, depth, alpha, beta and deadline; `None` means the deadline passed.
pub trait ABSearch<S: GameState>:
    Fn(&mut S, u8, EvalResult, EvalResult, Option<Instant>) -> Option<EvalResult>
{
    fn to_eval(self, depth: u8) -> impl Evaluation<S>
    where
        Self: Sized + Send,
    {
        move |state| self(&mut state.clone(), depth, Loss, Win, None).unwrap()
    }

    fn to_player(self, depth: u8) -> impl Player<S>
    where
        Self: Sized,
    {
        move |state| self.find_best(&mut state.clone(), depth, None).0
    }

    /// `first` is searched before the other moves, if it's one of them.
    fn find_best(
        &self,
        state: &mut S,
        depth: u8,
        first: Option<S::Choice>,
    ) -> (S::Choice, EvalResult) {
        self.find_best_until(state, depth, first, None).unwrap()
    }

    /// `find_best`, but `None` once `deadline` has passed, with `state` left as it was.
    fn find_best_until(
        &self,
        state: &mut S,
        depth: u8,
        first: Option<S::Choice>,
        deadline: Option<Instant>,
    ) -> Option<(S::Choice, EvalResult)> {
        let mut moves = state.candidate_moves();
        move_to_front(&mut moves, first);
        let mut alpha = Win;
        let mut alpha_move = moves[0];
        let beta = Loss;

        for game_move in moves {
            state.make_move(game_move);
            let score = self(state, depth, -beta, -alpha, deadline);
            state.undo();
            let score = score?;
            if score == beta {
                return Some((game_move, Win)); // beta cutoff
            }
            if score < alpha {
                alpha = score;
                alpha_move = game_move;
            }
        }

        Some((alpha_move, -alpha))
    }

    fn with_iterative(self, duration: Duration) -> impl Player<S>
    where
        Self: Sized,
    {
        move |state| {
            let start = Instant::now();
            let deadline = start + duration;
            let mut depth = 0;
            // Depth 0 runs without the deadline, so there's always a move.
            let (mut game_move, mut result) = self.find_best(&mut state.clone(), depth, None);
            // How long the last two depths took.
            let mut prev = Duration::ZERO;
            let mut last = start.elapsed();

            while !result.is_terminal() && start.elapsed() + next_depth_time(prev, last) < duration
            {
                let depth_start = Instant::now();
                let Some(found) = self.find_best_until(
                    &mut state.clone(),
                    depth + 1,
                    Some(game_move),
                    Some(deadline),
                ) else {
                    break; // out of time; keeps the last completed depth's move
                };
                depth += 1;
                (game_move, result) = found;
                (prev, last) = (last, depth_start.elapsed());
            }

            println!("Depth: {depth}, Choice: {game_move}, Result: {result}");
            game_move
        }
    }
}
impl<S: GameState, F> ABSearch<S> for F where
    F: Fn(&mut S, u8, EvalResult, EvalResult, Option<Instant>) -> Option<EvalResult>
{
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum EvalResult {
    Win,
    Loss,
    Draw,
    Score(f32),
}

pub fn alphabeta<S: GameState>(eval: impl Evaluation<S>) -> impl ABSearch<S> {
    fn recursive<S: GameState>(
        state: &mut S,
        depth: u8,
        mut alpha: EvalResult,
        beta: EvalResult,
        deadline: Option<Instant>,
        eval: &impl Evaluation<S>,
    ) -> Option<EvalResult> {
        if let Some(result) = EvalResult::terminal(state) {
            return Some(result);
        }

        if depth == 0 {
            return Some(eval(state));
        }

        // Past the leaves, which are most nodes, so the clock is read less often.
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return None;
        }

        for game_move in state.candidate_moves() {
            state.make_move(game_move);
            let score = recursive(state, depth - 1, -beta, -alpha, deadline, eval);
            state.undo();
            let score = score?;
            if score <= beta {
                return Some(-beta); // beta cutoff
            }
            if score < alpha {
                alpha = score;
            }
        }

        Some(-alpha)
    }

    move |state, depth, alpha, beta, deadline| {
        recursive(state, depth, alpha, beta, deadline, &eval)
    }
}

pub fn alphabeta_tt<S: GameState>(eval: impl Evaluation<S>) -> impl ABSearch<S> {
    struct SearchState<S: GameState, E> {
        eval: E,
        table: TTable<S::Choice>,
    }
    impl<S: GameState, E: Evaluation<S>> SearchState<S, E> {
        fn search(
            &mut self,
            state: &mut S,
            depth: u8,
            mut alpha: EvalResult,
            mut beta: EvalResult,
            deadline: Option<Instant>,
        ) -> Option<EvalResult> {
            if let Some(result) = EvalResult::terminal(state) {
                return Some(result);
            }

            if depth == 0 {
                return Some((self.eval)(state));
            }

            // Past the leaves, which are most nodes, so the clock is read less often.
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return None;
            }

            let alpha_orig = alpha;
            let state_hash = state.hash();
            let entry = self.table.get(state_hash);

            if let Some(entry) = entry
                && entry.depth >= depth
            {
                // Stored as this node's result; `alpha` and `beta` compare child scores, its negation.
                let value = -entry.value;
                match entry.bound {
                    TTBound::Exact => return Some(entry.value),
                    TTBound::Lower => {
                        if value <= beta {
                            return Some(-beta);
                        }
                        if value < alpha {
                            alpha = value;
                        }
                    }
                    TTBound::Upper => {
                        if value >= alpha {
                            return Some(-alpha);
                        }
                        if value > beta {
                            beta = value;
                        }
                    }
                }
            }

            let mut moves = state.candidate_moves();
            let hash_move = entry.and_then(|entry| entry.best_move);
            // Not found means a hash collision handed over another position's move.
            move_to_front(&mut moves, hash_move);
            let mut best_move = hash_move;

            for game_move in moves {
                state.make_move(game_move);
                let score = self.search(state, depth - 1, -beta, -alpha, deadline);
                state.undo();
                // Returns before any `store`, so an aborted search leaves the table alone.
                let score = score?;
                if score <= beta {
                    beta = -beta;
                    self.table
                        .store(state_hash, depth, beta, TTBound::Lower, Some(game_move));
                    return Some(beta); // beta cutoff
                }
                if score < alpha {
                    alpha = score;
                    best_move = Some(game_move);
                }
            }

            let bound = if alpha == alpha_orig {
                TTBound::Upper
            } else {
                TTBound::Exact
            };

            alpha = -alpha;
            self.table.store(state_hash, depth, alpha, bound, best_move);
            Some(alpha)
        }
    }

    // The search is `Fn`, so storing into the table needs a `RefCell`.
    let search = RefCell::new(SearchState {
        eval,
        table: TTable::new(),
    });
    move |state, depth, alpha, beta, deadline| {
        search.borrow_mut().search(state, depth, alpha, beta, deadline)
    }
}

/// Moves `first` to the front of `moves`, keeping the others in order; nothing if absent.
fn move_to_front<C: PartialEq>(moves: &mut [C], first: Option<C>) {
    if let Some(i) = first.and_then(|first| moves.iter().position(|m| *m == first)) {
        moves[..=i].rotate_right(1);
    }
}

/// The expected time of the next depth, given the last two depths took `prev` and `last`.
fn next_depth_time(prev: Duration, last: Duration) -> Duration {
    if prev.is_zero() {
        return last;
    }
    // Assumes each depth grows by the same factor as the last one did.
    let nanos = last.as_nanos() * last.as_nanos() / prev.as_nanos();
    Duration::from_nanos(nanos.try_into().unwrap_or(u64::MAX))
}

pub fn minimax<S: GameState>(eval: impl Evaluation<S> + Sync) -> impl Search<S> {
    fn recursive<S: GameState>(
        state: &mut S,
        depth: u8,
        eval: &(impl Evaluation<S> + Sync),
    ) -> EvalResult {
        if let Some(result) = EvalResult::terminal(state) {
            return result;
        }

        if depth == 0 {
            return eval(state);
        }

        -state
            .candidate_moves()
            .par_iter()
            .copied()
            .map_init(
                || state.clone(),
                |state, game_move| {
                    state.make_move(game_move);
                    let result = recursive(state, depth - 1, eval);
                    state.undo();
                    result
                },
            )
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    move |state: &mut S, depth| recursive(state, depth, &eval)
}
