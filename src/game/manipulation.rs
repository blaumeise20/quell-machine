use speedy2d::dimen::Vector2;
use crate::game::{direction::Direction, cells::{Cell, Grid}, cell_data::{WALL, SLIDE, MOVER, ORIENTATOR, TRASH, ENEMY, PULLER, PULLSHER, MIRROR, CROSSMIRROR, TRASHMOVER, SPEED, MOVLER, ONE_DIR, SLIDE_WALL, TRASHPULLER, GHOST, SUCKER}};

/// A force a cell is moved with.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveForce {
    Push,
    Pull,
    Swap,
}

/// The result when pushing a cell.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PushResult {
    Moved,
    NotMoved,
    Trashed,
}
impl PushResult {
    /// Returns whether the cell was moved.
    pub fn did_move(&self) -> bool {
        match self {
            PushResult::Moved => true,
            PushResult::NotMoved => false,
            PushResult::Trashed => true,
        }
    }

    /// Returns whether the cell was *not* trashed.
    pub fn did_move_survive(&self) -> bool {
        match self {
            PushResult::Moved => true,
            PushResult::NotMoved => false,
            PushResult::Trashed => false,
        }
    }
}

/// Checks if a cell can move in a certain direction with the given force.
pub fn can_move(cell: &Cell, direction: Direction, force: MoveForce) -> bool {
    match cell.id {
        WALL | GHOST => false,
        SLIDE | SLIDE_WALL if cell.direction.shrink(2) != direction.shrink(2) => false,
        ONE_DIR if cell.direction != direction => false,
        MIRROR if force == MoveForce::Swap && cell.direction.shrink(2) == direction.shrink(2) => false,
        CROSSMIRROR if force == MoveForce::Swap => false,
        _ => true,
    }
}

/// Checks if a cell is a trash in the direction it is being pushed.
pub fn is_trash(cell: &Cell, direction: Direction) -> bool {
    match cell.id {
        TRASH | ENEMY => true,
        TRASHMOVER if cell.direction == direction.flip() => true,
        TRASHPULLER if cell.direction == direction => true,
        SUCKER if cell.direction == direction.flip() => true,
        _ => false,
    }
}

/// Checks if a cell can be generated.
#[inline]
pub fn can_generate(cell: &Cell) -> bool {
    cell.id != GHOST
}

/// Pushes the specified cell in a direction. Returns whether the cell was moved.
/// You can also specify a replacement cell that should be put where the old one was.
pub fn push(grid: &mut Grid, x: isize, y: isize, dir: Direction, mut force: usize, pushing: Option<Cell>, setupdated: bool) -> PushResult {
    let mut tx = x;
    let mut ty = y;

    let orig_dir = dir;
    let mut dir = dir;

    // Check if the cell can be pushed.
    loop {
        if !grid.is_in_bounds(tx, ty) { return PushResult::NotMoved; }

        let cell = grid.get(tx, ty);
        if let Some(cell) = cell {
            if cell.id == MOVER || cell.id == PULLER || cell.id == PULLSHER || cell.id == TRASHMOVER || cell.id == SPEED || cell.id == MOVLER {
                if cell.direction == dir {
                    force += 1;
                }
                else if cell.direction == dir.flip() {
                    force -= 1;
                }
            }

            if is_trash(cell, dir) { break; }

            if !can_move(cell, dir, MoveForce::Push) {
                return PushResult::NotMoved;
            }

            let Vector2 { x: ox, y: oy } = dir.to_vector();
            tx += ox;
            ty += oy;
        }
        else {
            break;
        }

        if force == 0 { return PushResult::NotMoved; }
        if tx == x && ty == y && dir == orig_dir { break; }
    }

    // Push the cell and all following.
    // Works like this:
    //  >=#   replacement cell is air
    //  ^
    // replace cell with air and store the old cell in the replacement cell
    // then go forward one cell
    //   =#   replacement cell is mover
    //   ^
    // repeat ^
    //   >#   replacement cell is slide
    //   >=   replacement cell is push
    //   >=#
    // we moved forward one cell!

    dir = orig_dir;
    let mut x = x;
    let mut y = y;
    let mut next_cell = pushing;
    let mut push_result = PushResult::Trashed;
    loop {
        if let Some(ref mut cell) = next_cell {
            // Update mover cell `.updated`.
            if (cell.id == MOVER || cell.id == PULLER || cell.id == PULLSHER || cell.id == TRASHMOVER || cell.id == SPEED || cell.id == MOVLER) && cell.direction == dir && setupdated {
                cell.updated = true;
            }
        }

        if let Some(cell) = grid.get_mut(x, y) {
            // When trash then break.
            if cell.id == ENEMY {
                // Cell is deleted and enemy destroyed.
                grid.delete(x, y);
                break;
            }
            else if is_trash(cell, dir) {
                // Cell is trashed.
                break;
            }
        }

        // Push cell and store current one in next push replacement.
        push_result = PushResult::Moved;
        let old_cell = grid.take(x, y);
        grid.set_cell(x, y, next_cell);
        next_cell = old_cell;
        if tx == x && ty == y { break; }

        let Vector2 { x: ox, y: oy } = dir.to_vector();
        x += ox;
        y += oy;
    }

    push_result
}

/// Pulls the cell in a specific direction. Also pulls all cells behind it.
pub fn pull(grid: &mut Grid, x: isize, y: isize, dir: Direction) {
    let opposite_dir = dir.flip();
    let Vector2 { x: ox, y: oy } = dir.to_vector();
    let mut cx = x + ox;
    let mut cy = y + oy;
    let mut force = 1;

    // Pull the cells. Doesn't have replacement cells.
    // Works like this:
    // ##>
    // ## >
    // # #>
    //  ##>
    // Done!

    loop {
        let cell = grid.get_mut(cx - ox, cy - oy);
        if let Some(cell) = cell {
            if cell.id == MOVER || cell.id == PULLER || cell.id == PULLSHER || cell.id == TRASHMOVER || cell.id == SPEED || cell.id == MOVLER {
                if cell.direction == dir {
                    cell.updated = true;
                    force += 1;
                }
                else if cell.direction == dir.flip() {
                    force -= 1;
                }
            }

            if is_trash(cell, opposite_dir) || force == 0 || !can_move(cell, dir, MoveForce::Pull) {
                break;
            }

            let mut do_move = true;
            let old_cell = grid.get(cx, cy);
            if let Some(cell) = old_cell {
                if cell.id == ENEMY {
                    // cell is deleted and enemy destroyed
                    grid.delete(cx, cy);
                    do_move = false;
                }
                else if is_trash(cell, dir) {
                    // cell is trashed
                    do_move = false;
                }
            }

            let cell = grid.take(cx - ox, cy - oy).unwrap();
            if do_move {
                grid.set(cx, cy, cell);
            }

            cx -= ox;
            cy -= oy;
        }
        else {
            break;
        }
    }
}

// Checks if a cell can be rotated from a specific direction.
pub fn can_rotate(cell: &Cell, side: Direction) -> bool {
    match cell.id {
        WALL | GHOST => false,
        ORIENTATOR => false,
        SLIDE_WALL if (cell.direction - side).shrink(2) == Direction::Down => false,
        _ => true,
    }
}

// internal helper
#[inline]
unsafe fn rotate(cell: &mut Cell, dir: Direction, side: Direction) -> bool {
    if can_rotate(cell, side) {
        cell.direction = dir;
        true
    }
    else {
        false
    }
}

/// Rotates a cell by a specific amount.
pub fn rotate_by(grid: &mut Grid, x: isize, y: isize, dir: Direction, side: Direction) -> bool { unsafe {
    let cell = grid.get_mut(x, y);
    if let Some(cell) = cell {
        rotate(cell, cell.direction + dir, side)
    }
    else {
        false
    }
} }

/// Sets the direction of a cell.
pub fn rotate_to(grid: &mut Grid, x: isize, y: isize, dir: Direction, side: Direction) -> bool { unsafe {
    let cell = grid.get_mut(x, y);
    if let Some(cell) = cell {
        rotate(cell, dir, side)
    }
    else {
        false
    }
} }
