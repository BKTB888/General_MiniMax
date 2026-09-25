use general_minimax::{coordinate::Coordinate, state::GameState};

use crate::state::{BORDER, KInARowState, MapCoord};

pub fn human_kinrow<const K: u8, const NUM_P: u8>(state: &KInARowState<K, NUM_P>) -> MapCoord {
    use crossterm::{
        cursor::MoveTo,
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, MouseButton, MouseEventKind,
        },
        execute,
        terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
    };

    enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(stdout, EnableMouseCapture).unwrap();

    let (Coordinate(min_r, min_c), Coordinate(max_r, max_c)) = state.bounds();

    let (term_width, term_height) = size().unwrap_or((80, 24));
    let board_width = (max_c - min_c + 1 + 2 * BORDER) as u16;
    let board_height = (max_r - min_r + 1 + 2 * BORDER) as u16;
    let col_offset = (term_width.saturating_sub(board_width)) / 2;
    let row_offset = (term_height.saturating_sub(board_height)) / 2;

    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();

    for (i, line) in state.to_string().lines().enumerate() {
        execute!(stdout, MoveTo(col_offset, row_offset + i as u16)).unwrap();
        print!("{line}");
    }
    let _ = std::io::Write::flush(&mut stdout);

    execute!(stdout, MoveTo(col_offset, row_offset + board_height + 1)).unwrap();
    print!("Click a cell:");
    let _ = std::io::Write::flush(&mut stdout);

    let result = loop {
        if let Ok(Event::Mouse(m)) = event::read()
            && m.kind == MouseEventKind::Down(MouseButton::Left)
        {
            let term_col = m.column as i16 - col_offset as i16;
            let term_row = m.row as i16 - row_offset as i16;

            let board_col = term_col + (min_c - BORDER);
            let board_row = (max_r + BORDER) - term_row;

            let coord = Coordinate(board_row, board_col);

            if state.is_valid(coord) {
                break coord;
            } else {
                execute!(stdout, MoveTo(col_offset, row_offset + board_height + 2)).unwrap();
                print!("Invalid move at {coord:?}, try again\r");
                let _ = std::io::Write::flush(&mut stdout);
            }
        }
    };

    execute!(stdout, DisableMouseCapture).unwrap();
    disable_raw_mode().unwrap();
    result
}
