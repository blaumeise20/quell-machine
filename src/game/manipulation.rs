use speedy2d::dimen::Vector2;
use crate::game::{direction::Direction, cells::{grid, Cell}, cell_data::{WALL, SLIDE, MOVER, ORIENTATOR, TRASH, ENEMY, PULLER, PULLSHER, MIRROR, CROSSMIRROR, TRASHMOVER, SPEED, MOVLER, ONE_DIR, SLIDE_WALL, TRASHPULLER, GHOST, SUCKER}};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveForce {
    Mover,
    Puller,
    Mirror,
}

pub fn can_move(cell: &Cell, direction: Direction, force: MoveForce) -> bool {
    match cell.id {
        WALL | GHOST => false,
        SLIDE | SLIDE_WALL if cell.direction.shrink(2) != direction.shrink(2) => false,
        ONE_DIR if cell.direction != direction => false,
        MIRROR if force == MoveForce::Mirror && cell.direction.shrink(2) == direction.shrink(2) => false,
        CROSSMIRROR if force == MoveForce::Mirror => false,
        _ => true,
    }
}

#[inline]
pub fn is_trash(cell: &Cell, direction: Direction) -> bool {
    match cell.id {
        TRASH | ENEMY => true,
        TRASHMOVER if cell.direction == direction.flip() => true,
        TRASHPULLER if cell.direction == direction => true,
        SUCKER if cell.direction == direction.flip() => true,
        _ => false,
    }
}

pub fn can_generate(cell: &Cell) -> bool {
    cell.id != GHOST
}

pub fn push(x: isize, y: isize, dir: Direction, mut force: usize, pushing: Option<Cell>) -> bool { unsafe {
    let mut tx = x;
    let mut ty = y;
    let Vector2 { x: ox, y: oy } = dir.to_vector();

    loop {
        if !grid.is_in_bounds(tx, ty) { return false; }

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

            if !can_move(cell, dir, MoveForce::Mover) {
                return false;
            }

            tx += ox;
            ty += oy;
        }
        else {
            break;
        }

        if force == 0 { return false; }
    }

    let mut did_survive = true;
    while tx != x || ty != y {
        tx -= ox;
        ty -= oy;

        let mut cell = grid.take(tx, ty).unwrap();
        if (cell.id == MOVER || cell.id == PULLER || cell.id == PULLSHER || cell.id == TRASHMOVER || cell.id == SPEED || cell.id == MOVLER) && cell.direction == dir {
            cell.updated = true;
        }

        if let Some(cell) = grid.get(tx + ox, ty + oy) {
            if cell.id == ENEMY {
                // cell is deleted and enemy destroyed
                grid.delete(tx + ox, ty + oy);
                did_survive = false;
                continue;
            }
            else if is_trash(cell, dir) {
                // cell is trashed
                did_survive = false;
                continue;
            }
        }

        grid.set(tx + ox, ty + oy, cell);
    }

    if let Some(cell) = pushing {
        grid.set(x, y, cell);
    }

    did_survive
} }

pub fn pull(x: isize, y: isize, dir: Direction) { unsafe {
    let opposite_dir = dir.flip();
    let Vector2 { x: ox, y: oy } = dir.to_vector();
    let mut cx = x + ox;
    let mut cy = y + oy;
    let mut force = 1;

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

            if is_trash(cell, opposite_dir) || force == 0 || !can_move(cell, dir, MoveForce::Puller) {
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
} }

pub fn can_rotate(cell: &Cell, side: Direction) -> bool {
    match cell.id {
        WALL | GHOST => false,
        ORIENTATOR => false,
        SLIDE_WALL if (cell.direction - side).shrink(2) == Direction::Down => false,
        _ => true,
    }
}

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

pub fn rotate_by(x: isize, y: isize, dir: Direction, side: Direction) -> bool { unsafe {
    let cell = grid.get_mut(x, y);
    if let Some(cell) = cell {
        rotate(cell, cell.direction + dir, side)
    }
    else {
        false
    }
} }

pub fn rotate_to(x: isize, y: isize, dir: Direction, side: Direction) -> bool { unsafe {
    let cell = grid.get_mut(x, y);
    if let Some(cell) = cell {
        rotate(cell, dir, side)
    }
    else {
        false
    }
} }
