use super::direction::Direction;

pub const DEFAULT_GRID_WIDTH: usize = 100;
pub const DEFAULT_GRID_HEIGHT: usize = 100;

pub static mut grid: Grid = Grid::new_const(DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT);
pub static mut initial: Grid = Grid::new_const(DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT);

pub type CellType = u16;

static mut DUMMY_CELL: Option<Cell> = None;

#[derive(Debug)]
pub struct Cell {
    pub id: CellType,
    pub direction: Direction,
    pub updated: bool,
}

impl Cell {
    pub fn new(id: CellType, direction: Direction) -> Self {
        Cell {
            id,
            direction,
            updated: false,
        }
    }

    pub fn copy(&self) -> Self {
        Cell {
            id: self.id,
            direction: self.direction,
            updated: false,
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
        assert!(self.cells.is_empty());
        self.cells = vec![None; self.width * self.height];
    }

    pub fn is_in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn get(&self, x: isize, y: isize) -> &Option<Cell> {
        if self.is_in_bounds(x, y) {
            &self.cells[y as usize * self.width + x as usize]
        }
        else {
            &None
        }
    }

    pub fn get_unchecked(&self, x: isize, y: isize) -> &Option<Cell> {
        &self.cells[y as usize * self.width + x as usize]
    }

    pub fn get_mut(&mut self, x: isize, y: isize) -> &mut Option<Cell> {
        if self.is_in_bounds(x, y) {
            &mut self.cells[y as usize * self.width + x as usize]
        }
        else {
            unsafe { &mut DUMMY_CELL }
        }
    }

    pub fn set(&mut self, x: isize, y: isize, cell: Cell) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = Some(cell);
        }
    }

    pub fn set_cell(&mut self, x: isize, y: isize, cell: Option<Cell>) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = cell;
        }
    }

    pub fn delete(&mut self, x: isize, y: isize) {
        if self.is_in_bounds(x, y) {
            self.cells[y as usize * self.width + x as usize] = None;
        }
    }

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

    pub fn for_each_mut(&mut self, mut f: impl FnMut(isize, isize, &mut Cell)) {
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(cell) = self.cells[y * self.width + x].as_mut() {
                    f(x as isize, y as isize, cell);
                }
            }
        }
    }

    pub fn for_each_dir(&mut self, dir: Direction, mut f: impl FnMut(isize, isize, &mut Cell)) {
        if dir == Direction::Right || dir == Direction::Up {
            for y in (0..self.height).rev() {
                for x in (0..self.width).rev() {
                    if let Some(cell) = self.cells[y * self.width + x].as_mut() {
                        f(x as isize, y as isize, cell);
                    }
                }
            }
        }
        else {
            for y in 0..self.height {
                for x in 0..self.width {
                    if let Some(cell) = self.cells[y * self.width + x].as_mut() {
                        f(x as isize, y as isize, cell);
                    }
                }
            }
        }
    }
}
