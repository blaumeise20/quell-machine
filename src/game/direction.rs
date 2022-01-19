use std::{ops::{Add, Sub, Rem, AddAssign, SubAssign}, fmt::Display};

use speedy2d::dimen::Vector2;

/// A direction of a cell.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
	Right,
	Down,
	Left,
	Up,
}

impl Direction {
    /// Turns the direction into degrees.
    #[inline]
	pub fn to_degrees(self) -> f32 {
		match self {
			Direction::Right => 0.0,
			Direction::Down => 90.0,
			Direction::Left => 180.0,
			Direction::Up => 270.0,
		}
	}

    /// Turns the direction into radians.
    #[inline(always)]
	pub fn to_radians(self) -> f32 {
		self.to_degrees().to_radians()
	}

    /// Turns the direction into a vector.
    #[inline]
	pub fn to_vector(self) -> Vector2<isize> {
		match self {
			Direction::Right => Vector2::new(1, 0),
			Direction::Down => Vector2::new(0, -1),
			Direction::Left => Vector2::new(-1, 0),
			Direction::Up => Vector2::new(0, 1),
		}
	}

    /// Rotates the direction 180 degrees.
    #[inline]
	pub fn flip(self) -> Direction {
		match self {
			Direction::Right => Direction::Left,
			Direction::Down => Direction::Up,
			Direction::Left => Direction::Right,
			Direction::Up => Direction::Down,
		}
	}

    /// Rotates the direction clockwise.
    #[inline]
    pub fn rotate_left(self) -> Direction {
        match self {
            Direction::Right => Direction::Up,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Up => Direction::Left,
        }
    }

    /// Rotates the direction counter-clockwise.
    #[inline]
    pub fn rotate_right(self) -> Direction {
        match self {
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Up => Direction::Right,
        }
    }

    /// Reduces the direction to a radius/range.
    #[inline(always)]
    pub fn shrink(self, radius: u8) -> Direction {
        self % radius
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Right => write!(f, "Right"),
            Direction::Down => write!(f, "Down"),
            Direction::Left => write!(f, "Left"),
            Direction::Up => write!(f, "Up"),
        }
    }
}

impl From<i32> for Direction {
	fn from(i: i32) -> Direction {
		match i & 3 {
			0 => Direction::Right,
			1 => Direction::Down,
			2 => Direction::Left,
			3 => Direction::Up,
			_ => panic!("Invalid direction: {}", i),
		}
	}
}

impl From<u8> for Direction {
	fn from(i: u8) -> Direction {
		match i & 3 {
			0 => Direction::Right,
			1 => Direction::Down,
			2 => Direction::Left,
			3 => Direction::Up,
			_ => panic!("Invalid direction: {}", i),
		}
	}
}

impl From<usize> for Direction {
    fn from(i: usize) -> Direction {
        match i & 3 {
            0 => Direction::Right,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Up,
            _ => panic!("Invalid direction: {}", i),
        }
    }
}

impl From<Direction> for i32 {
	fn from(d: Direction) -> Self {
		match d {
			Direction::Right => 0,
			Direction::Down => 1,
			Direction::Left => 2,
			Direction::Up => 3,
		}
	}
}

impl From<Direction> for u8 {
	fn from(d: Direction) -> Self {
		match d {
			Direction::Right => 0,
			Direction::Down => 1,
			Direction::Left => 2,
			Direction::Up => 3,
		}
	}
}

impl From<Direction> for usize {
	fn from(d: Direction) -> Self {
		match d {
			Direction::Right => 0,
			Direction::Down => 1,
			Direction::Left => 2,
			Direction::Up => 3,
		}
	}
}

impl Add for Direction {
	type Output = Direction;

    #[inline(always)]
	fn add(self, rhs: Self) -> Self::Output {
		((u8::from(self) + u8::from(rhs)) & 3).into()
	}
}

impl Add<u8> for Direction {
	type Output = Direction;

    #[inline(always)]
	fn add(self, rhs: u8) -> Self::Output {
		((u8::from(self) + rhs) & 3).into()
	}
}

impl AddAssign<u8> for Direction {
    #[inline(always)]
    fn add_assign(&mut self, rhs: u8) {
        *self = *self + rhs;
    }
}

impl Sub for Direction {
	type Output = Direction;

    #[inline(always)]
	fn sub(self, rhs: Self) -> Self::Output {
		(((u8::from(self) as i8 - u8::from(rhs) as i8) & 3) as u8).into()
	}
}

impl Sub<u8> for Direction {
	type Output = Direction;

    #[inline(always)]
	fn sub(self, rhs: u8) -> Self::Output {
		(((u8::from(self) as i8 - rhs as i8) & 3) as u8).into()
	}
}

impl SubAssign<u8> for Direction {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: u8) {
        *self = *self - rhs;
    }
}

impl Rem<u8> for Direction {
    type Output = Direction;

    #[inline(always)]
    fn rem(self, rhs: u8) -> Self::Output {
        ((u8::from(self) % rhs) & 3).into()
    }
}
