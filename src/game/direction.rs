use std::{ops::{Add, Sub, Rem, AddAssign, SubAssign}, fmt::Display, hint::unreachable_unchecked};

use speedy2d::dimen::Vector2;

/// A direction of a cell.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
	Right = 0,
	Down = 1,
	Left = 2,
	Up = 3,
}

impl Direction {
    /// Turns the direction into degrees.
    #[inline(always)]
	pub fn to_degrees(self) -> f32 {
        self as u8 as f32 * 90.0
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
    #[inline(always)]
	pub fn flip(self) -> Direction {
        ((self as u8 + 2) & 3).into()
	}

    /// Rotates the direction clockwise.
    #[inline(always)]
    pub fn rotate_left(self) -> Direction {
        ((self as u8 + 3) & 3).into()
    }

    /// Rotates the direction counter-clockwise.
    #[inline(always)]
    pub fn rotate_right(self) -> Direction {
        ((self as u8 + 1) & 3).into()
    }

    /// Reduces the direction to a radius/range.
    #[inline(always)]
    pub fn shrink(self, radius: u8) -> Direction {
        self % radius
    }
}

impl Display for Direction {
    #[inline]
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
    #[inline]
	fn from(i: i32) -> Direction {
		match i & 3 {
			0 => Direction::Right,
			1 => Direction::Down,
			2 => Direction::Left,
			3 => Direction::Up,
			_ => unsafe { unreachable_unchecked() },
		}
	}
}

impl From<u8> for Direction {
    #[inline]
	fn from(i: u8) -> Direction {
		match i & 3 {
			0 => Direction::Right,
			1 => Direction::Down,
			2 => Direction::Left,
			3 => Direction::Up,
			_ => unsafe { unreachable_unchecked() },
		}
	}
}

impl From<usize> for Direction {
    #[inline]
    fn from(i: usize) -> Direction {
        match i & 3 {
            0 => Direction::Right,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Up,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

impl From<Direction> for i32 {
    #[inline(always)]
	fn from(d: Direction) -> Self {
		d as u8 as i32
	}
}

impl From<Direction> for u8 {
    #[inline(always)]
	fn from(d: Direction) -> Self {
		d as u8
	}
}

impl From<Direction> for usize {
    #[inline(always)]
	fn from(d: Direction) -> Self {
		d as u8 as usize
	}
}

impl Add for Direction {
	type Output = Direction;

    #[inline(always)]
	fn add(self, rhs: Self) -> Self::Output {
		((self as u8 + rhs as u8) & 3).into()
	}
}

impl Add<u8> for Direction {
	type Output = Direction;

    #[inline(always)]
	fn add(self, rhs: u8) -> Self::Output {
		((self as u8 + rhs) & 3).into()
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
		(((self as i8 - rhs as i8) & 3) as u8).into()
	}
}

impl Sub<u8> for Direction {
	type Output = Direction;

    #[inline(always)]
	fn sub(self, rhs: u8) -> Self::Output {
		(((self as i8 - rhs as i8) & 3) as u8).into()
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
        ((self as u8 % rhs) & 3).into()
    }
}
