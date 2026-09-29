use general_minimax::{coordinate::Coordinate, state::GameState};

use crate::{
    map::MapCoord,
    state::{BORDER, CELL_HEIGHT, CELL_WIDTH, FiveInRowState},
};

pub fn human_five_in_row(state: &FiveInRowState) -> MapCoord {
    use crossterm::{
        cursor::MoveTo,
        event::{
            self, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind,
        },
        execute,
        terminal::enable_raw_mode,
    };

    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(stdout, EnableMouseCapture).unwrap();

    let (Coordinate(_, min_c), Coordinate(max_r, _)) = state.bounds();
    let (mut col_offset, mut row_offset, mut message_row) = draw(state, "Click a cell");

    let result = loop {
        match event::read() {
            Ok(Event::Mouse(m)) if m.kind == MouseEventKind::Down(MouseButton::Left) => {
                let term_col = m.column as i16 - col_offset as i16;
                let term_row = m.row as i16 - row_offset as i16;

                // A click on a grid line counts for the cell left of it. `div_euclid` keeps a
                // click left of or above the board off the board.
                let board_col = term_col.div_euclid(CELL_WIDTH) + (min_c - BORDER);
                let board_row = (max_r + BORDER) - term_row.div_euclid(CELL_HEIGHT);

                let coord = Coordinate(board_row, board_col);

                if state.is_valid(coord) {
                    // The board is drawn again only on the next turn, after the opponent's move.
                    let mut next = state.clone();
                    next.make_move(coord);
                    draw(&next, "Waiting for the other player...");
                    break coord;
                }
                execute!(stdout, MoveTo(col_offset, message_row)).unwrap();
                print!("Invalid move at {coord:?}, try again\r");
                let _ = std::io::Write::flush(&mut stdout);
            }
            // Changing the font size resizes the grid, so the board has to be centred again.
            Ok(Event::Resize(..)) => {
                (col_offset, row_offset, message_row) = draw(state, "Click a cell")
            }
            // Raw mode turns Ctrl-C into a key press instead of a SIGINT.
            Ok(Event::Key(k))
                if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                restore();
                std::process::exit(130);
            }
            _ => {}
        }
    };

    restore();
    result
}

/// Clears the terminal and draws `state` centred with `prompt` below it. Returns the board's
/// column and row offset, and the row free for messages.
fn draw(state: &FiveInRowState, prompt: &str) -> (u16, u16, u16) {
    use crossterm::{
        cursor::MoveTo,
        execute,
        terminal::{Clear, ClearType, size},
    };

    let mut stdout = std::io::stdout();
    let (Coordinate(min_r, min_c), Coordinate(max_r, max_c)) = state.bounds();

    let (term_width, term_height) = size().unwrap_or((80, 24));
    // The last column has no grid line after it.
    let board_width = ((max_c - min_c + 1 + 2 * BORDER) * CELL_WIDTH - 1) as u16;
    let board_height = ((max_r - min_r + 1 + 2 * BORDER) * CELL_HEIGHT) as u16;
    let col_offset = (term_width.saturating_sub(board_width)) / 2;
    let row_offset = (term_height.saturating_sub(board_height)) / 2;

    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();

    for (i, line) in state.to_string().lines().enumerate() {
        execute!(stdout, MoveTo(col_offset, row_offset + i as u16)).unwrap();
        print!("{line}");
    }

    execute!(stdout, MoveTo(col_offset, row_offset + board_height + 1)).unwrap();
    print!("{prompt}");
    let _ = std::io::Write::flush(&mut stdout);

    (col_offset, row_offset, row_offset + board_height + 2)
}

/// Undoes the raw mode and mouse capture `human_five_in_row` turns on.
fn restore() {
    use crossterm::{event::DisableMouseCapture, execute, terminal::disable_raw_mode};

    execute!(std::io::stdout(), DisableMouseCapture).unwrap();
    disable_raw_mode().unwrap();
}
