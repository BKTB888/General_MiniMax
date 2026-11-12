use general_minimax::player::evals::Score;

use super::HiveState;
use crate::{
    bug::Bug,
    hex_grid::{Hex, adjacent},
    node::Color,
};

/// Scores a position for the player to move, with the default aggression.
pub fn eval<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>(
    state: &HiveState<MOSQUITO, LADYBUG, PILLBUG>,
) -> Score {
    BasicEvaluator::default().evaluate(state)
}

/// nokamute's evaluation: how many pieces each side can move, and how surrounded each queen
/// is.
#[derive(Clone, Copy)]
pub struct BasicEvaluator {
    aggression: i32,
    queen_liberty_factor: i32,
    movable_queen_value: i32,
    movable_bug_factor: i32,
    unplayed_bug_factor: i32,
    /// For a pillbug that can save its queen, or room to place one.
    pillbug_defense_bonus: i32,
}

impl BasicEvaluator {
    /// The score for the player to move.
    pub fn evaluate<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>(
        &self,
        board: &HiveState<MOSQUITO, LADYBUG, PILLBUG>,
    ) -> Score {
        let mut immovable = board.find_cut_vertexes();
        let to_move = board.to_move();
        let (me, them) = (to_move as usize, to_move.other());
        let mut score = 0;
        let mut pillbug_defense = [false; 2];
        let mut queen_score = [0; 2];

        for bug in Bug::ALL {
            let unplaced = i32::from(board.remaining[me][bug as usize])
                - i32::from(board.remaining[them][bug as usize]);
            score += unplaced * self.unplayed_bug_factor * self.value(bug);
        }

        for &hex in board.occupied_hexes.iter().flatten() {
            let node = board.node(hex);
            let mut bug_score = self.value(node.bug());
            let mut pillbug_powers = node.bug() == Bug::Pillbug;
            let mut crawler = node.bug().crawler();
            if node.bug() == Bug::Mosquito {
                // Valued as what it can move as now.
                bug_score = 0;
                crawler = true;
                if node.is_stacked() {
                    bug_score = self.value(Bug::Beetle);
                } else {
                    for adj in board.occupied_adjacent(hex) {
                        let bug = board.node(adj).bug();
                        if bug != Bug::Queen {
                            bug_score = self.value(bug);
                        }
                        if bug == Bug::Pillbug {
                            pillbug_powers = true;
                        }
                        if !bug.crawler() {
                            crawler = false;
                        }
                    }
                }
            }

            // A crawler with nowhere to slide is as stuck as a pinned piece.
            if crawler && board.slidable_adjacent(hex, hex).next().is_none() {
                immovable.set(hex);
            }

            if node.is_stacked() {
                bug_score *= 2;
            }

            let color = node.color() as usize;
            let friendly_queen = board.queens[color];
            if adjacent(friendly_queen).contains(&hex) {
                // Filling its own queen's liberty, for good unless it can leave.
                if immovable.get(hex) && !node.is_stacked() {
                    queen_score[color] -= self.queen_liberty_factor;
                } else {
                    queen_score[color] -= self.queen_liberty_factor / 2;
                }
                if pillbug_powers && board.node(friendly_queen).clipped_height() == 1 {
                    // The freest hex it could throw its queen to.
                    let best_escape = adjacent(hex)
                        .into_iter()
                        .map(|lib| {
                            if board.occupied(lib) {
                                0
                            } else {
                                count_liberties(board, friendly_queen, lib)
                            }
                        })
                        .max()
                        .unwrap_or(0);
                    if best_escape > 2 {
                        pillbug_defense[color] = true;
                    }
                }
            }

            let enemy = node.color().other();
            let enemy_queen = board.queens[enemy];
            if adjacent(enemy_queen).contains(&hex) {
                // Worth nothing there, so valuable bugs don't settle on a liberty.
                bug_score = 0;
                // A little extra, since it won't choose to leave.
                queen_score[enemy] -= self.queen_liberty_factor * 12 / 10;
                if pillbug_powers {
                    // The least free hex it could throw the enemy queen to.
                    let best_unescape = adjacent(hex)
                        .into_iter()
                        .map(|lib| {
                            if board.occupied(lib) {
                                6
                            } else {
                                count_liberties(board, enemy_queen, lib)
                            }
                        })
                        .min()
                        .unwrap_or(6);
                    if best_unescape < 3 {
                        queen_score[enemy] = -self.queen_liberty_factor;
                    }
                }
            }

            if !node.is_stacked() && immovable.get(hex) {
                // Pinned, so worth nothing.
                continue;
            }

            bug_score *= self.movable_bug_factor;
            if node.color() != to_move {
                bug_score = -bug_score;
                // Low aggression weighs the opponent's mobility above its own.
                if self.aggression == 1 {
                    bug_score *= 2;
                } else if self.aggression == 2 {
                    bug_score = bug_score * 3 / 2;
                }
            }
            score += bug_score;
        }

        let mut pillbug_defense_score = self.pillbug_defense_bonus
            * (i32::from(pillbug_defense[me]) - i32::from(pillbug_defense[them]));

        // Room to place a pillbug next to the queen counts for half.
        if PILLBUG {
            let placeable_defense = [Color::White, Color::Black].map(|color| {
                let queen = board.queens[color as usize];
                board.node(queen).clipped_height() == 1
                    && board.remaining[color as usize][Bug::Pillbug as usize] > 0
                    && adjacent(queen)
                        .into_iter()
                        .any(|lib| placeable(board, lib, color))
            });
            pillbug_defense_score += self.pillbug_defense_bonus / 2
                * (i32::from(placeable_defense[me]) - i32::from(placeable_defense[them]));
        }

        let queen_score = queen_score[me] - queen_score[them];
        (queen_score + pillbug_defense_score + score) as Score
    }
}

impl BasicEvaluator {
    /// `aggression` from 1 to 5, clamped: higher presses the opponent's queen harder.
    pub fn new(aggression: u8) -> Self {
        let aggression = i32::from(aggression.clamp(1, 5));
        Self {
            aggression,
            queen_liberty_factor: aggression * 10,
            movable_queen_value: aggression * 4,
            movable_bug_factor: 2,
            unplayed_bug_factor: 1,
            pillbug_defense_bonus: aggression * 40,
        }
    }

    fn value(&self, bug: Bug) -> i32 {
        // nokamute's, mostly made up: "All I know is that ants are good."
        match bug {
            Bug::Queen => self.movable_queen_value,
            Bug::Ant => 7,
            Bug::Beetle => 6,
            Bug::Grasshopper => 2,
            Bug::Spider => 2,
            // Only for unplaced ones; on the board it's valued as what it copies.
            Bug::Mosquito => 8,
            Bug::Ladybug => 6,
            Bug::Pillbug => 5,
        }
    }
}

impl Default for BasicEvaluator {
    fn default() -> Self {
        Self::new(3)
    }
}

/// The empty neighbors of `hex`, counting `origin` as empty.
fn count_liberties<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>(
    board: &HiveState<MOSQUITO, LADYBUG, PILLBUG>,
    origin: Hex,
    hex: Hex,
) -> i32 {
    6 - board.occupied_neighbors(origin, hex, &adjacent(hex)).count_ones() as i32
}

/// Whether `color` could place on `hex`, ignoring whether it's next to its own pieces.
pub(super) fn placeable<const MOSQUITO: bool, const LADYBUG: bool, const PILLBUG: bool>(
    board: &HiveState<MOSQUITO, LADYBUG, PILLBUG>,
    hex: Hex,
    color: Color,
) -> bool {
    !adjacent(hex)
        .into_iter()
        .any(|adj| board.occupied(adj) && board.node(adj).color() != color)
}
