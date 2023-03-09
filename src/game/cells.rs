use std::mem;

use super::{direction::Direction, cell_data::CELL_DATA};

pub const DEFAULT_GRID_WIDTH: usize = 100;
pub const DEFAULT_GRID_HEIGHT: usize = 100;

pub type CellType = u8;

static mut DUMMY_CELL: Option<Cell> = None;

/// Represents a cell on a grid.
///
/// # Internal layout
///
/// The cell is stored as a single byte. It uses the following layout:
/// ```txt
/// UCCCCCDD
/// ```
/// Where:
/// - `U`: The updated flag. If set, the cell has been updated this tick.
/// - `C`: The cell type.
/// - `D`: The direction.
#[derive(Debug)]
pub struct Cell(u8);

impl Cell {
    /// Creates a new cell.
    #[inline(always)]
    pub fn new(id: CellType, direction: Direction) -> Self {
        Cell((id << 2) | direction as u8)
    }

    /// Gets the cell type.
    #[inline(always)]
    pub fn id(&self) -> CellType {
        (self.0 & 124) >> 2
    }

    /// Gets the direction.
    #[inline(always)]
    pub fn direction(&self) -> Direction {
        (self.0 & 3).into()
    }

    /// Sets the direction.
    #[inline(always)]
    pub fn set_direction(&mut self, direction: Direction) {
        self.0 = (self.0 & 252) | (direction as u8);
    }

    /// Gets the updated flag.
    #[inline(always)]
    pub fn updated(&self) -> bool {
        self.0 & 128 != 0
    }

    /// Sets the updated flag.
    #[inline(always)]
    pub fn set_updated(&mut self, updated: bool) {
        self.0 = (self.0 & 127) | ((updated as u8) << 7);
    }

    pub fn looks_like(&self, other: &Cell) -> bool {
        if self.0 & 124 != other.0 & 124 {
            return false;
        }
        let max_rot = CELL_DATA.iter().find(|cd| cd.id == (self.0 & 124) >> 2).unwrap().sides as u8;
        (self.0 & 3) % max_rot == (other.0 & 3) % max_rot
    }
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        Cell(self.0 & 127)
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Cell) -> bool {
        self.0 & 127 == other.0 & 127
    }
}
impl Eq for Cell {}

/// A whole grid of cells.
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Option<Cell>>,
    pub tick_count: u32,
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
            tick_count: 0,
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
            tick_count: 0,
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
    #[inline(always)]
    pub fn get<'a, 'b: 'a>(&'a self, x: isize, y: isize) -> &'b Option<Cell> {
        if self.is_in_bounds(x, y) {
            // SAFETY: We checked the bounds above.
            unsafe { mem::transmute(self.cells.get_unchecked(y as usize * self.width + x as usize)) }
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
    #[inline(always)]
    pub fn get_mut<'a, 'b: 'a>(&'a mut self, x: isize, y: isize) -> &'b mut Option<Cell> {
        if self.is_in_bounds(x, y) {
            unsafe { mem::transmute(self.cells.get_unchecked_mut(y as usize * self.width + x as usize)) }
        }
        else {
            unsafe { &mut DUMMY_CELL }
        }
    }

    /// Overrides the cell at the coordinate.
    #[inline(always)]
    pub fn set(&mut self, x: isize, y: isize, cell: Cell) {
        if self.is_in_bounds(x, y) {
            unsafe { *self.cells.get_unchecked_mut(y as usize * self.width + x as usize) = Some(cell); }
        }
    }

    /// Overrides the cell at the coordinate.
    /// Can also pass `None` to remove the cell.
    #[inline(always)]
    pub fn set_cell(&mut self, x: isize, y: isize, cell: Option<Cell>) {
        if self.is_in_bounds(x, y) {
            unsafe { *self.cells.get_unchecked_mut(y as usize * self.width + x as usize) = cell; }
        }
    }

    /// Tries to set a cell at the specified index.
    #[inline(always)]
    pub fn try_set(&mut self, ix: usize, cell: Option<Cell>) -> bool {
        if ix < self.width * self.height {
            unsafe { *self.cells.get_unchecked_mut(ix) = cell; }
            true
        }
        else {
            false
        }
    }

    /// Replaces the cell at the coordinate with air.
    #[inline(always)]
    pub fn delete(&mut self, x: isize, y: isize) {
        if self.is_in_bounds(x, y) {
            unsafe { *self.cells.get_unchecked_mut(y as usize * self.width + x as usize) = None; }
        }
    }

    /// Takes out the cell at the coordinate, leaving air.
    #[inline(always)]
    pub fn take(&mut self, x: isize, y: isize) -> Option<Cell> {
        if self.is_in_bounds(x, y) {
            unsafe { self.cells.get_unchecked_mut(y as usize * self.width + x as usize).take() }
        }
        else {
            None
        }
    }

    /// Iterates over every cell in the grid.
    pub fn for_each(&self, mut f: impl FnMut(isize, isize, Option<&Cell>)) {
        unsafe {
            for y in 0..self.height {
                for x in 0..self.width {
                    f(x as isize, y as isize, self.cells.get_unchecked(y * self.width + x).as_ref());
                }
            }
        }
    }

    pub fn has_same_cells(&self, other: &Grid) -> bool {
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.get(x as isize, y as isize);
                let other_cell = other.get(x as isize, y as isize);
                match (cell, other_cell) {
                    (Some(c), Some(oc)) => {
                        if !c.looks_like(oc) { return false; }
                    },
                    (None, None) => {},
                    _ => return false,
                }
            }
        }
        true
    }
}

impl PartialEq for Grid {
    fn eq(&self, other: &Grid) -> bool {
        self.width == other.width && self.height == other.height && self.cells == other.cells
    }
}
impl Eq for Grid {}
