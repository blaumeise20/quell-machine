use std::mem;

use super::direction::Direction;

pub const DEFAULT_GRID_WIDTH: usize = 100;
pub const DEFAULT_GRID_HEIGHT: usize = 100;

pub type CellType = u16;

static mut DUMMY_CELL: Option<Cell> = None;

/// Represents a cell on a grid.
#[derive(Debug)]
pub struct Cell {
    pub id: CellType,
    pub direction: Direction,
    pub updated: bool,
    pub contained_cell: Option<(CellType, Direction)>,
}

impl Cell {
    /// Creates a new cell.
    pub fn new(id: CellType, direction: Direction) -> Self {
        Cell {
            id,
            direction,
            updated: false,
            contained_cell: None,
        }
    }

    /// Creates a copy of the cell,
    /// without copying the updated state.
    pub fn copy(&self) -> Self {
        Cell {
            id: self.id,
            direction: self.direction,
            updated: false,
            contained_cell: self.contained_cell,
        }
    }
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        self.copy()
    }
}

/// A whole grid of cells.
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Option<Cell>>,
}

impl Grid {
    /// Creates a new uninitialized grid.
    /// You have to call `Grid::init` before using it.
    pub const fn new_const(width: usize, height: usize) -> Self {
        assert!(width > 0);
        assert!(height > 0);

        Grid {
            width,
            height,
            cells: Vec::new(),
        }
    }

    /// Creates a new initialized grid.
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width > 0);
        assert!(height > 0);

        let mut g = Grid {
            width,
            height,
            cells: Vec::new(),
        };
        g.init();
        g
    }

    /// Initializes the grid.
    /// Fills the cell collection with `None`.
    /// This is called automatically by `Grid::new`.
    pub fn init(&mut self) {
        assert!(self.width > 0);
        assert!(self.height > 0);
        assert!(self.cells.is_empty());
        self.cells = vec![None; self.width * self.height];
    }

    /// Checks if a given coordinate is inside the grid bounds.
    #[inline(always)]
    pub fn is_in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    /// Gets a immutable reference to the cell at the coordinate.
    /// Returns `None` if the coordinate is outside the grid bounds.
    #[inline]
    pub fn get<'a, 'b: 'a>(&'a self, x: isize, y: isize) -> &'b Option<Cell> {
        if self.is_in_bounds(x, y) {
            unsafe { mem::transmute(&self.cells[y as usize * self.width + x as usize]) }
        }
        else {
            &None
        }
    }

    /// Gets a immutable reference to the cell at the coordinate without checking bounds.
    /// Might panic if the coordinate is outside the grid bounds.
    #[doc(hidden)]
    #[inline(always)]
    pub fn get_unchecked(&self, x: isize, y: isize) -> &Option<Cell> {
        &self.cells[y as usize * self.width + x as usize]
    }

    /// Gets a mutable reference to the cell at the coordinate.
    /// Returns `None` if the coordinate is outside the grid bounds.
    #[inline]
    pub fn get_mut<'a, 'b: 'a>(&'a mut self, x: isize, y: isize) -> &'b mut Option<Cell> {
        if self.is_in_bounds(x, y) {
            unsafe { mem::transmute(&mut self.cells[y as usize * self.width + x as usize]) }
        }
        else {
            unsafe { &mut DUMMY_CELL }
        }
    }

    /// Overrides the cell at the coordinate.
    #[inline(always)]
    pub fn set(&mut self, x: isize, y: isize, cell: Cell) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = Some(cell);
        }
    }

    /// Overrides the cell at the coordinate.
    /// Can also pass `None` to remove the cell.
    #[inline(always)]
    pub fn set_cell(&mut self, x: isize, y: isize, cell: Option<Cell>) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = cell;
        }
    }

    /// Replaces the cell at the coordinate with air.
    #[inline(always)]
    pub fn delete(&mut self, x: isize, y: isize) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = None;
        }
    }

    /// Takes out the cell at the coordinate, leaving air.
    #[inline]
    pub fn take(&mut self, x: isize, y: isize) -> Option<Cell> {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize].take()
        }
        else {
            None
        }
    }

    /// Iterates over every cell in the grid.
    pub fn for_each(&self, mut f: impl FnMut(isize, isize, Option<&Cell>)) {
        for y in 0..self.height {
            for x in 0..self.width {
                f(x as isize, y as isize, self.cells[y * self.width + x].as_ref());
            }
        }
    }
}
