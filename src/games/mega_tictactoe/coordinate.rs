use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Mul};
use std::str::FromStr;

pub type MapInt = i16;

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct Coordinate(pub MapInt, pub MapInt);

impl Add for Coordinate {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Coordinate(self.0 + other.0, self.1 + other.1)
    }
}

impl Add<(MapInt, MapInt)> for Coordinate {
    type Output = Self;

    fn add(self, other: (MapInt, MapInt)) -> Self::Output {
        Coordinate(self.0 + other.0, self.1 + other.1)
    }
}

impl Add<Coordinate> for (MapInt, MapInt) {
    type Output = Coordinate;

    fn add(self, rhs: Coordinate) -> Self::Output {
        Coordinate(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl From<(MapInt, MapInt)> for Coordinate {
    fn from(coord: (MapInt, MapInt)) -> Self {
        Coordinate(coord.0, coord.1)
    }
}

impl AddAssign<(MapInt, MapInt)> for Coordinate {
    fn add_assign(&mut self, rhs: (MapInt, MapInt)) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct ParseCoordinateError(String);
impl FromStr for Coordinate {

    type Err = ParseCoordinateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().trim_start_matches('(').trim_end_matches(')');

        let (x, y) = s.split_once(',')
            .ok_or_else(|| ParseCoordinateError(format!("expected comma-separated pair, got: {s}")))?;

        let x = x.trim().parse()
            .map_err(|e| ParseCoordinateError(format!("invalid x: {e}")))?;
        let y = y.trim().parse()
            .map_err(|e| ParseCoordinateError(format!("invalid y: {e}")))?;

        Ok(Coordinate(x, y))
    }
}

impl Mul<MapInt> for Coordinate {
    type Output = Coordinate;

    fn mul(self, rhs: MapInt) -> Self::Output {
        (rhs * self.0, rhs * self.1).into()
    }
}
impl Mul<Coordinate> for MapInt {
    type Output = Coordinate;

    fn mul(self, Coordinate(r, c): Coordinate) -> Self::Output {
        (self * r, self * c).into()
    }
}