use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Mul};
use std::str::FromStr;

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct Coordinate<X, Y>(pub X, pub Y);

impl<X: Add<Output = X>, Y: Add<Output = Y>> Add for Coordinate<X, Y> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Coordinate(self.0 + other.0, self.1 + other.1)
    }
}

impl<X: Add<Output = X>, Y: Add<Output = Y>> Add<(X, Y)> for Coordinate<X, Y> {
    type Output = Self;

    fn add(self, other: (X, Y)) -> Self::Output {
        Coordinate(self.0 + other.0, self.1 + other.1)
    }
}

impl<X: Add<Output = X>, Y: Add<Output = Y>> Add<Coordinate<X, Y>> for (X, Y) {
    type Output = Coordinate<X, Y>;

    fn add(self, rhs: Coordinate<X, Y>) -> Self::Output {
        Coordinate(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<X, Y> From<(X, Y)> for Coordinate<X, Y> {
    fn from((x, y): (X, Y)) -> Self {
        Coordinate(x, y)
    }
}

impl<X: AddAssign, Y: AddAssign> AddAssign<(X, Y)> for Coordinate<X, Y> {
    fn add_assign(&mut self, rhs: (X, Y)) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl<X: AddAssign, Y: AddAssign> AddAssign for Coordinate<X, Y> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl<X: Display, Y: Display> Display for Coordinate<X, Y> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct ParseCoordinateError(String);
impl<X: FromStr, Y: FromStr> FromStr for Coordinate<X, Y> {
    type Err = ParseCoordinateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().trim_start_matches('(').trim_end_matches(')');

        let (x, y) = s.split_once(',').ok_or_else(|| {
            ParseCoordinateError(format!("expected comma-separated pair, got: {s}"))
        })?;

        let x = x
            .trim()
            .parse()
            .map_err(|_| ParseCoordinateError(format!("invalid x: {}", x)))?;
        let y = y
            .trim()
            .parse()
            .map_err(|_| ParseCoordinateError(format!("invalid y: {}", y)))?;

        Ok(Coordinate(x, y))
    }
}

impl<X, Y, Z: Mul<Y, Output = Y> + Mul<X, Output = X> + Clone> Mul<Z> for Coordinate<X, Y> {
    type Output = Coordinate<X, Y>;

    fn mul(self, rhs: Z) -> Self::Output {
        Coordinate(rhs.clone() * self.0, rhs * self.1)
    }
}
