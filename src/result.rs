use std::fmt::{Display, Formatter};
use colored::{Color, ColoredString, Colorize};

#[derive(Ord, Eq, PartialEq, PartialOrd, Debug)]
pub enum GameResult {
    Player(u8),
    Draw
}

impl Display for GameResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_colored())
    }
}

impl GameResult {

    fn get_colored(&self) -> ColoredString {
        if let GameResult::Player(player) = self {
            format!("Player {}", player + 1).color(
                match player {
                    0 => Color::Red,
                    1 => Color::Blue,
                    2 => Color::Green,
                    3 => Color::Magenta,
                    _ => color_from_id(*player),
                }
            )
        } else {
            "Draw".color(Color::White)
        }
    }

    pub fn print_result(&self) {
        if let GameResult::Player(_) = self {
            println!("{} won!", self);
        } else {
            println!("It's a draw!");
        }
    }
}

pub fn get_player_color(player: u8) -> Color {
    match player {
        0 => Color::Red,
        1 => Color::Blue,
        2 => Color::Green,
        3 => Color::Magenta,
        _ => color_from_id(player),
    }
}

fn color_from_id(id: u8) -> Color {
    let hue = (id as u16 * 137) % 360; // golden angle spacing
    hsv_to_rgb(hue as f32, 0.7, 0.9)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h as i32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r1 + m) * 255.0) as u8;
    let g = ((g1 + m) * 255.0) as u8;
    let b = ((b1 + m) * 255.0) as u8;

    Color::TrueColor { r, g, b }
}