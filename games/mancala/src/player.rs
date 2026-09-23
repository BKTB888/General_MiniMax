use std::io::Write;
use crossterm::style::Stylize;
use general_minimax::state::GameState;
use crate::state::MancalaState;

pub fn human(board: &MancalaState) -> u8 {
    println!("{}", board);
    let valid_choices = board.candidate_moves();
    println!("Available choices: {:?}", valid_choices);
    let choice= loop {
        print!(
            "{}",
            if board.current_player() == 0 {
                "Player 1's choice: ".red()
            } else {
                "Player 2's choice: ".green()
            }
        );
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .unwrap_or_else(
                |_| {
                    println!("Failed to read line. Please try again.");
                    100
                }
            );

        let choice = input.trim().parse();

        if choice.is_err() {
            println!("Invalid choice. Please try again.");
            continue;
        }

        let choice = choice.unwrap();

        if valid_choices.contains(&choice) {
            break choice;
        } else {
            println!("Invalid choice. Please try again.");
        }
    };
    let mut copy = board.clone();
    copy.make_move(choice);
    println!("Result: {}\n", copy);
    choice
}