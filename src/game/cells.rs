use super::direction::Direction;

pub const DEFAULT_GRID_WIDTH: usize = 100;
pub const DEFAULT_GRID_HEIGHT: usize = 100;

pub type CellType = u16;

static mut DUMMY_CELL: Option<Cell> = None;

#[derive(Debug)]
pub struct Cell {
    pub id: CellType,
    pub direction: Direction,
    pub updated: bool,
    pub contained_cell: Option<(CellType, Direction)>,
}

impl Cell {
    pub fn new(id: CellType, direction: Direction) -> Self {
        Cell {
            id,
            direction,
            updated: false,
            contained_cell: None,
        }
    }

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

#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Option<Cell>>,
}

impl Grid {
    pub const fn new_const(width: usize, height: usize) -> Self {
        assert!(width > 0);
        assert!(height > 0);

        Grid {
            width,
            height,
            cells: Vec::new(),
        }
    }

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

    pub fn init(&mut self) {
        assert!(self.width > 0);
        assert!(self.height > 0);
        assert!(self.cells.is_empty());
        self.cells = vec![None; self.width * self.height];
    }

    #[inline(always)]
    pub fn is_in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    #[inline]
    pub fn get<'a, 'b: 'a>(&'a self, x: isize, y: isize) -> &'b Option<Cell> {
        if self.is_in_bounds(x, y) {
            unsafe { &*(&self.cells[y as usize * self.width + x as usize] as *const _) }
        }
        else {
            &None
        }
    }

    #[inline(always)]
    pub fn get_unchecked(&self, x: isize, y: isize) -> &Option<Cell> {
        &self.cells[y as usize * self.width + x as usize]
    }

    #[inline]
    pub fn get_mut<'a, 'b: 'a>(&'a mut self, x: isize, y: isize) -> &'b mut Option<Cell> {
        if self.is_in_bounds(x, y) {
            unsafe { &mut *(&mut self.cells[y as usize * self.width + x as usize] as *mut _) }
        }
        else {
            unsafe { &mut DUMMY_CELL }
        }
    }

    #[inline(always)]
    pub fn set(&mut self, x: isize, y: isize, cell: Cell) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = Some(cell);
        }
    }

    #[inline(always)]
    pub fn set_cell(&mut self, x: isize, y: isize, cell: Option<Cell>) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = cell;
        }
    }

    #[inline(always)]
    pub fn delete(&mut self, x: isize, y: isize) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = None;
        }
    }

    #[inline]
    pub fn take(&mut self, x: isize, y: isize) -> Option<Cell> {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize].take()
        }
        else {
            None
        }
    }

    pub fn for_each(&self, mut f: impl FnMut(isize, isize, Option<&Cell>)) {
        for y in 0..self.height {
            for x in 0..self.width {
                f(x as isize, y as isize, self.cells[y * self.width + x].as_ref());
            }
        }
    }
}
