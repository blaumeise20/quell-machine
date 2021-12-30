use std::collections::HashSet;

use super::{cells::grid, manipulation::{push, rotate_by, rotate_to, pull, MoveForce, can_move}, direction::Direction, cell_data::{MOVER, GENERATOR, ROTATOR_CCW, ROTATOR_CW, ORIENTATOR, PULLER, PULLSHER, MIRROR}};

static UPDATE_DIRECTIONS: [Direction; 4] = [
    Direction::Right,
    Direction::Left,
    Direction::Down,
    Direction::Up,
];

pub fn update() {
    unsafe {
        let mut cells = HashSet::new();

        for y in 0..grid.height as isize {
            for x in 0..grid.width as isize {
                if let Some(cell) = grid.get_mut(x, y) {
                    cell.updated = false;
                    cells.insert(cell.id);
                }
            }
        }

        if cells.contains(&MIRROR) { do_mirrors(); }
        if cells.contains(&GENERATOR) { do_gens(); }
        if cells.contains(&ROTATOR_CW) || cells.contains(&ROTATOR_CCW) { do_rotators(); }
        if cells.contains(&ORIENTATOR) { do_orientators(); }
        if cells.contains(&PULLSHER) { do_pullshers(); }
        if cells.contains(&PULLER) { do_pullers(); }
        if cells.contains(&MOVER) { do_movers(); }
    }
}

unsafe fn do_mirrors() {
    grid.for_each_mut(|x, y, cell| {
        if cell.id == MIRROR && cell.direction.shrink(2) == Direction::Right && !cell.updated {
            cell.updated = true;
            let cell_left = grid.get(x - 1, y);
            let cell_right = grid.get(x + 1, y);
            let left_movable = if let Some(cell) = cell_left {
                can_move(cell, Direction::Right, MoveForce::Mirror)
            } else { true };
            let right_movable = if let Some(cell) = cell_right {
                can_move(cell, Direction::Left, MoveForce::Mirror)
            } else { true };
            if left_movable && right_movable {
                let cell_left = grid.take(x - 1, y);
                let cell_right = grid.take(x + 1, y);
                grid.set_cell(x - 1, y, cell_right);
                grid.set_cell(x + 1, y, cell_left);
            }
        }
    });
    grid.for_each_mut(|x, y, cell| {
        if cell.id == MIRROR && cell.direction.shrink(2) == Direction::Down && !cell.updated {
            cell.updated = true;
            let cell_up = grid.get(x, y + 1);
            let cell_down = grid.get(x, y - 1);
            let up_movable = if let Some(cell) = cell_up {
                can_move(cell, Direction::Down, MoveForce::Mirror)
            } else { true };
            let down_movable = if let Some(cell) = cell_down {
                can_move(cell, Direction::Up, MoveForce::Mirror)
            } else { true };
            if up_movable && down_movable {
                let cell_up = grid.take(x, y + 1);
                let cell_down = grid.take(x, y - 1);
                grid.set_cell(x, y + 1, cell_down);
                grid.set_cell(x, y - 1, cell_up);
            }
        }
    });
}

unsafe fn do_gens() {
    for dir in UPDATE_DIRECTIONS {
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == GENERATOR && cell.direction == dir && !cell.updated {
                cell.updated = true;
                let push_offset = cell.direction.to_vector();
                let px = x + push_offset.x;
                let py = y + push_offset.y;

                let cell_offset = cell.direction.flip().to_vector();
                let cx = x + cell_offset.x;
                let cy = y + cell_offset.y;

                if let Some(cell) = grid.get(cx, cy) {
                    push(px, py, dir, 1, Some(cell.copy()));
                }
            }
        });
    }
}

unsafe fn do_rotators() {
    grid.for_each_mut(|x, y, cell| {
        if !cell.updated {
            if cell.id == ROTATOR_CW {
                cell.updated = true;
                rotate_by(x + 1, y, Direction::Down);
                rotate_by(x, y - 1, Direction::Down);
                rotate_by(x - 1, y, Direction::Down);
                rotate_by(x, y + 1, Direction::Down);
            }
            else if cell.id == ROTATOR_CCW {
                cell.updated = true;
                rotate_by(x + 1, y, Direction::Up);
                rotate_by(x, y - 1, Direction::Up);
                rotate_by(x - 1, y, Direction::Up);
                rotate_by(x, y + 1, Direction::Up);
            }
        }
    });
}

unsafe fn do_orientators() {
    grid.for_each_mut(|x, y, cell| {
        if cell.id == ORIENTATOR && !cell.updated {
            cell.updated = true;
            rotate_to(x + 1, y, cell.direction);
            rotate_to(x, y - 1, cell.direction);
            rotate_to(x - 1, y, cell.direction);
            rotate_to(x, y + 1, cell.direction);
        }
    });
}

unsafe fn do_pullshers() {
    for dir in UPDATE_DIRECTIONS {
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == PULLSHER && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if push(x, y, dir, 1, None) {
                    let off = dir.to_vector();
                    pull(x - off.x, y - off.y, dir);
                }
            }
        });
    }
}

unsafe fn do_pullers() {
    for dir in UPDATE_DIRECTIONS {
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == PULLER && cell.direction == dir && !cell.updated {
                cell.updated = true;
                let off = cell.direction.to_vector();
                if grid.get(x + off.x, y + off.y).is_none() {
                    pull(x, y, dir);
                }
            }
        });
    }
}

unsafe fn do_movers() {
    for dir in UPDATE_DIRECTIONS {
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == MOVER && cell.direction == dir && !cell.updated {
                cell.updated = true;
                push(x, y, dir, 0, None);
            }
        });
    }
}
