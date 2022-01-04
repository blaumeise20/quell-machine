use std::collections::HashSet;

use super::{cells::grid, manipulation::{push, rotate_by, rotate_to, pull, MoveForce, can_move, is_trash, can_generate}, direction::Direction, cell_data::{MOVER, GENERATOR, ROTATOR_CCW, ROTATOR_CW, ORIENTATOR, PULLER, PULLSHER, MIRROR, CROSSMIRROR, TRASHMOVER, SPEED, GENERATOR_CW, GENERATOR_CCW, TRASHPULLER, STONE}};

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
        if cells.contains(&CROSSMIRROR) { do_crossmirrors(); }
        if cells.contains(&GENERATOR) { do_gens(); }
        if cells.contains(&GENERATOR_CW) || cells.contains(&GENERATOR_CCW) { do_angled_gens(); }
        if cells.contains(&ROTATOR_CW) || cells.contains(&ROTATOR_CCW) { do_rotators(); }
        if cells.contains(&ORIENTATOR) { do_orientators(); }
        if cells.contains(&STONE) { do_stones(); }
        if cells.contains(&PULLSHER) { do_pullshers(); }
        if cells.contains(&TRASHPULLER) { do_trashpullers(); }
        if cells.contains(&PULLER) { do_pullers(); }
        if cells.contains(&TRASHMOVER) { do_trashmovers(); }
        if cells.contains(&MOVER) { do_movers(); }
        if cells.contains(&SPEED) { do_speeds(); }
    }
}

unsafe fn do_mirrors() {
    grid.for_each_mut(|x, y, cell| {
        if cell.id == MIRROR && cell.direction.shrink(2) == Direction::Right && !cell.updated {
            cell.updated = true;
            let cell_left = grid.get_mut(x - 1, y);
            let cell_right = grid.get_mut(x + 1, y);
            if let Some(cell) = cell_left { if !can_move(cell, Direction::Right, MoveForce::Mirror) { return; } }
            if let Some(cell) = cell_right { if !can_move(cell, Direction::Left, MoveForce::Mirror) { return; } }

            let cell_left = cell_left.take();
            grid.set_cell(x - 1, y, cell_right.take());
            grid.set_cell(x + 1, y, cell_left);
        }
    });
    grid.for_each_mut(|x, y, cell| {
        if cell.id == MIRROR && cell.direction.shrink(2) == Direction::Down && !cell.updated {
            cell.updated = true;
            let cell_up = grid.get_mut(x, y + 1);
            let cell_down = grid.get_mut(x, y - 1);
            if let Some(cell) = cell_up { if !can_move(cell, Direction::Down, MoveForce::Mirror) { return; } }
            if let Some(cell) = cell_down { if !can_move(cell, Direction::Up, MoveForce::Mirror) { return; } }

            let cell_up = cell_up.take();
            grid.set_cell(x, y + 1, cell_down.take());
            grid.set_cell(x, y - 1, cell_up);
        }
    });
}

unsafe fn do_crossmirrors() {
    grid.for_each_mut(|x, y, cell| {
        if cell.id == CROSSMIRROR && !cell.updated {
            cell.updated = true;
            let cell_left = grid.get_mut(x - 1, y);
            let cell_right = grid.get_mut(x + 1, y);
            let left_movable = if let Some(cell) = cell_left {
                can_move(cell, Direction::Right, MoveForce::Mirror)
            } else { true };
            let right_movable = if let Some(cell) = cell_right {
                can_move(cell, Direction::Left, MoveForce::Mirror)
            } else { true };
            if left_movable && right_movable {
                let cell_left = cell_left.take();
                grid.set_cell(x - 1, y, cell_right.take());
                grid.set_cell(x + 1, y, cell_left);
            }

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
                grid.set_cell(x, y + 1, grid.take(x, y - 1));
                grid.set_cell(x, y - 1, cell_up);
            }
        }
    });
}

unsafe fn do_gens() {
    for dir in UPDATE_DIRECTIONS {
        let push_offset = dir.to_vector();
        let cell_offset = dir.flip().to_vector();
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == GENERATOR && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if let Some(cell) = grid.get(x + cell_offset.x, y + cell_offset.y) {
                    if can_generate(cell) {
                        push(x + push_offset.x, y + push_offset.y, dir, 1, Some(cell.copy()));
                    }
                }
            }
        });
    }
}

unsafe fn do_angled_gens() {
    for dir in UPDATE_DIRECTIONS {
        let cell_offset = dir.flip().to_vector();
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == GENERATOR_CW && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if let Some(cell) = grid.get(x + cell_offset.x, y + cell_offset.y) {
                    let mut cell = cell.copy();
                    cell.direction = cell.direction.rotate_right();
                    let push_offset = dir.rotate_right().to_vector();
                    push(x + push_offset.x, y + push_offset.y, dir.rotate_right(), 1, Some(cell));
                }
            }
            else if cell.id == GENERATOR_CCW && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if let Some(cell) = grid.get(x + cell_offset.x, y + cell_offset.y) {
                    let mut cell = cell.copy();
                    cell.direction = cell.direction.rotate_left();
                    let push_offset = dir.rotate_left().to_vector();
                    push(x + push_offset.x, y + push_offset.y, dir.rotate_left(), 1, Some(cell));
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
                rotate_by(x + 1, y, Direction::Down, Direction::Left);
                rotate_by(x, y - 1, Direction::Down, Direction::Up);
                rotate_by(x - 1, y, Direction::Down, Direction::Right);
                rotate_by(x, y + 1, Direction::Down, Direction::Down);
            }
            else if cell.id == ROTATOR_CCW {
                cell.updated = true;
                rotate_by(x + 1, y, Direction::Up, Direction::Left);
                rotate_by(x, y - 1, Direction::Up, Direction::Up);
                rotate_by(x - 1, y, Direction::Up, Direction::Right);
                rotate_by(x, y + 1, Direction::Up, Direction::Down);
            }
        }
    });
}

unsafe fn do_orientators() {
    grid.for_each_mut(|x, y, cell| {
        if cell.id == ORIENTATOR && !cell.updated {
            cell.updated = true;
            rotate_to(x + 1, y, cell.direction, Direction::Left);
            rotate_to(x, y - 1, cell.direction, Direction::Up);
            rotate_to(x - 1, y, cell.direction, Direction::Right);
            rotate_to(x, y + 1, cell.direction, Direction::Down);
        }
    });
}

unsafe fn do_stones() {
    for dir in UPDATE_DIRECTIONS {
        grid.for_each_dir(dir.rotate_right(), |x, y, cell| {
            if cell.id == STONE && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if !push(x, y, dir.rotate_right(), 1, None) {
                    // complex logic lol

                    let down = dir.rotate_right();
                    let off_right = dir.to_vector();
                    let off_down = down.to_vector();
                    let off_left = dir.flip().to_vector();

                    let cell_right = grid.get(x + off_right.x, y + off_right.y);
                    let cell_left = grid.get(x + off_left.x, y + off_left.y);

                    let cell_right_down = grid.get(x + off_right.x + off_down.x, y + off_right.y + off_down.y);
                    let cell_left_down = grid.get(x + off_left.x + off_down.x, y + off_left.y + off_down.y);
                    let can_move_right = grid.is_in_bounds(x + off_right.x + off_down.x, y + off_right.y + off_down.y) && cell_right_down.is_none();
                    let can_move_left = grid.is_in_bounds(x + off_left.x + off_down.x, y + off_left.y + off_down.y) && cell_left_down.is_none();
                    let has_free = can_move_right || can_move_left;
                    if !has_free { return; }

                    let prefered_dir;
                    if cell_right.is_none() && can_move_right {
                        if cell_left.is_some() && can_move_left {
                            prefered_dir = Some(dir);
                        }
                        else {
                            prefered_dir = None;
                        }
                    }
                    else if cell_left.is_none() {
                        if can_move_left {
                            prefered_dir = Some(dir.flip());
                        }
                        else {
                            return;
                        }
                    }
                    else {
                        let cell_right = cell_right.as_ref();
                        let cell_left = cell_left.as_ref();
                        if let Some(cell_left) = cell_left {
                            if (is_trash(cell_left, dir.flip()) || !can_move(cell_left, dir.flip(), MoveForce::Mover)) && can_move_right {
                                prefered_dir = Some(dir);
                            }
                            else {
                                prefered_dir = None;
                            }
                        }
                        else if let Some(cell_right) = cell_right {
                            if (is_trash(cell_right, dir) || !can_move(cell_right, dir, MoveForce::Mover)) && can_move_left {
                                prefered_dir = Some(dir.flip());
                            }
                            else {
                                prefered_dir = None;
                            }
                        }
                        else {
                            prefered_dir = None;
                        }
                    }

                    if let Some(dir) = prefered_dir {
                        let off = dir.to_vector();
                        if push(x, y, dir, 1, None) {
                            push(x + off.x, y + off.y, down, 1, None);
                        }
                    }
                    else if can_move_left && !can_move_right {
                        if push(x, y, dir.flip(), 1, None) {
                            push(x, y, down, 1, None);
                        }
                    }
                    else if push(x, y, dir, 1, None) {
                        push(x, y, down, 1, None);
                    }
                }
            }
        });
    }
}

unsafe fn do_pullshers() {
    for dir in UPDATE_DIRECTIONS {
        let off = dir.to_vector();
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == PULLSHER && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if push(x, y, dir, 1, None) {
                    pull(x - off.x, y - off.y, dir);
                }
            }
        });
    }
}

unsafe fn do_trashpullers() {
    for dir in UPDATE_DIRECTIONS {
        let off = dir.flip().to_vector();
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == TRASHPULLER && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if let Some(pushed) = grid.get(x + off.x, y + off.y) {
                    if can_move(pushed, dir, MoveForce::Puller) && !is_trash(pushed, dir) {
                        grid.delete(x + off.x, y + off.y);
                        if grid.get(x - off.x, y - off.y).is_none() {
                            pull(x, y, dir);
                        }
                    }
                }
            }
        });
    }
}

unsafe fn do_pullers() {
    for dir in UPDATE_DIRECTIONS {
        let off = dir.to_vector();
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == PULLER && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if grid.get(x + off.x, y + off.y).is_none() {
                    pull(x, y, dir);
                }
            }
        });
    }
}

unsafe fn do_trashmovers() {
    for dir in UPDATE_DIRECTIONS {
        let off = dir.to_vector();
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == TRASHMOVER && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if let Some(pushed) = grid.get(x + off.x, y + off.y) {
                    if !can_move(pushed, dir, MoveForce::Mover) || is_trash(cell, dir) {
                        return;
                    }
                }
                grid.delete(x + off.x, y + off.y);
                push(x, y, dir, 0, None);
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

unsafe fn do_speeds() {
    for dir in UPDATE_DIRECTIONS {
        let off = dir.to_vector();
        grid.for_each_dir(dir, |x, y, cell| {
            if cell.id == SPEED && cell.direction == dir && !cell.updated {
                cell.updated = true;
                if grid.get(x + off.x, y + off.y).is_none() {
                    push(x, y, dir, 0, None);
                }
            }
        });
    }
}
