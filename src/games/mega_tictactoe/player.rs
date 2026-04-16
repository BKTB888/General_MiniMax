use crate::games::mega_tictactoe::coordinate::Coordinate;
use crate::games::mega_tictactoe::state::KInARowState;
use crate::state::GameState;

pub fn human_kinrow<const K: u8, const NUM_P: u8>(state: &KInARowState<K, NUM_P>) -> Coordinate {
    use crossterm::{
        cursor::MoveTo,
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseButton, MouseEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    };

    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(stdout, EnableMouseCapture).unwrap();

    let cells = state.cells();
    let max_r = cells.keys().map(|c| c.0).max().unwrap_or(0);
    let min_r = cells.keys().map(|c| c.0).min().unwrap_or(0);
    let min_c = cells.keys().map(|c| c.1).min().unwrap_or(0);
    let max_c = cells.keys().map(|c| c.1).max().unwrap_or(0);
    let bx: i16 = 2;

    let (term_width, term_height) = size().unwrap_or((80, 24));
    let board_width = (max_c - min_c + 1 + 2 * bx) as u16;
    let board_height = (max_r - min_r + 1 + 2 * bx) as u16;
    let col_offset = (term_width.saturating_sub(board_width)) / 2;
    let row_offset = (term_height.saturating_sub(board_height)) / 2;

    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();

    for (i, line) in format!("{}", state).lines().enumerate() {
        execute!(stdout, MoveTo(col_offset, row_offset + i as u16)).unwrap();
        print!("{}", line);
    }
    let _ = std::io::Write::flush(&mut stdout);

    execute!(stdout, MoveTo(col_offset, row_offset + board_height + 1)).unwrap();
    print!("Click a cell:");
    let _ = std::io::Write::flush(&mut stdout);

    let result = loop {
        if let Ok(Event::Mouse(m)) = event::read() {
            if m.kind == MouseEventKind::Down(MouseButton::Left) {
                let term_col = m.column as i16 - col_offset as i16;
                let term_row = m.row as i16 - row_offset as i16;

                let board_col = term_col + (min_c - bx);
                let board_row = (max_r + bx) - term_row;

                let coord = Coordinate(board_row, board_col);

                if state.is_valid(coord) {
                    break coord;
                } else {
                    execute!(stdout, MoveTo(col_offset, row_offset + board_height + 2)).unwrap();
                    print!("Invalid move at {:?}, try again\r", coord);
                    let _ = std::io::Write::flush(&mut stdout);
                }
            }
        }
    };

    execute!(stdout, DisableMouseCapture).unwrap();
    disable_raw_mode().unwrap();
    result
}