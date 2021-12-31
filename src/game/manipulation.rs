use speedy2d::dimen::Vector2;
use crate::game::{direction::Direction, cells::{grid, Cell}, cell_data::{WALL, SLIDE, MOVER, ORIENTATOR, TRASH, ENEMY, PULLER, PULLSHER, MIRROR, CROSSMIRROR, TRASHMOVER, SPEED, MOVLER}};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveForce {
    Mover,
    Puller,
    Mirror,
}

pub fn can_move(cell: &Cell, direction: Direction, force: MoveForce) -> bool {
    match cell.id {
        WALL => false,
        SLIDE if cell.direction.shrink(2) != direction.shrink(2) => false,
        MIRROR if force == MoveForce::Mirror && cell.direction.shrink(2) == direction.shrink(2) => false,
        CROSSMIRROR if force == MoveForce::Mirror => false,
        _ => true,
    }
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

            if cell.id == TRASH || cell.id == ENEMY { break; }

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

    while tx != x || ty != y {
        tx -= ox;
        ty -= oy;

        let mut cell = grid.take(tx, ty).unwrap();
        if (cell.id == MOVER || cell.id == PULLER || cell.id == PULLSHER || cell.id == TRASHMOVER || cell.id == SPEED || cell.id == MOVLER) && cell.direction == dir {
            cell.updated = true;
        }

        if let Some(cell) = grid.get(tx + ox, ty + oy) {
            if cell.id == TRASH {
                // cell is trashed
                continue;
            }
            else if cell.id == ENEMY {
                // cell is deleted and enemy destroyed
                grid.delete(tx + ox, ty + oy);
                continue;
            }
        }

        grid.set(tx + ox, ty + oy, cell);
    }

    if let Some(cell) = pushing {
        grid.set(x, y, cell);
    }

    true
} }

pub fn pull(x: isize, y: isize, dir: Direction) { unsafe {
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

            if cell.id == TRASH || cell.id == ENEMY || force == 0 || !can_move(cell, dir, MoveForce::Puller) {
                break;
            }

            grid.set(cx, cy, grid.take(cx - ox, cy - oy).unwrap());

            cx -= ox;
            cy -= oy;
        }
        else {
            break;
        }
    }
} }

pub fn can_rotate(cell: &Cell) -> bool {
    #[allow(clippy::match_like_matches_macro)]
    match cell.id {
        WALL => false,
        ORIENTATOR => false,
        _ => true,
    }
}

unsafe fn rotate(cell: &mut Cell, dir: Direction) -> bool {
    if can_rotate(cell) {
        cell.direction = dir;
        true
    }
    else {
        false
    }
}

pub fn rotate_by(x: isize, y: isize, dir: Direction) -> bool { unsafe {
    let cell = grid.get_mut(x, y);
    if let Some(cell) = cell {
        rotate(cell, cell.direction + dir)
    }
    else {
        false
    }
} }

pub fn rotate_to(x: isize, y: isize, dir: Direction) -> bool { unsafe {
    let cell = grid.get_mut(x, y);
    if let Some(cell) = cell {
        rotate(cell, dir)
    }
    else {
        false
    }
} }
