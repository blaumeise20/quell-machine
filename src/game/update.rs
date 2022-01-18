use std::collections::HashSet;

use super::{cells::{Grid, Cell}, manipulation::{push, rotate_by, rotate_to, pull, MoveForce, can_move, is_trash, can_generate}, direction::Direction, cell_data::{MOVER, GENERATOR, ROTATOR_CCW, ROTATOR_CW, ORIENTATOR, PULLER, PULLSHER, MIRROR, CROSSMIRROR, TRASHMOVER, SPEED, GENERATOR_CW, GENERATOR_CCW, TRASHPULLER, STONE, REPLICATOR, SUCKER, GENERATOR_CROSS, MAILBOX, POSTOFFICE, PHYSICAL_GENERATOR}};

static UPDATE_DIRECTIONS: [Direction; 4] = [
    Direction::Right,
    Direction::Left,
    Direction::Down,
    Direction::Up,
];

macro_rules! loop_each {
    (for $x:ident, $y:ident, $name:ident in $grid:expr; $code:block) => {
        for $y in 0..$grid.height as isize {
            for $x in 0..$grid.width as isize {
                if let Some($name) = $grid.get_mut($x, $y) {
                    $code
                }
            }
        }
    };
}

macro_rules! loop_each_dir {
    (for $dir:ident $({ $($s:stmt;)* })?, $x:ident, $y:ident, $name:ident in $grid:expr; $code:block) => {
        for $dir in UPDATE_DIRECTIONS {
            $($( $s )*)?
            if $dir == Direction::Right || $dir == Direction::Up {
                let mut $y = $grid.height as isize - 1;
                while $y >= 0 {
                    let mut $x = $grid.width as isize - 1;
                    while $x >= 0 {
                        if let Some($name) = $grid.get_mut($x, $y) {
                            $code
                        }
                        $x -= 1;
                    }
                    $y -= 1;
                }
            }
            else {
                for $y in 0..$grid.height as isize {
                    for $x in 0..$grid.width as isize {
                        if let Some($name) = $grid.get_mut($x, $y) {
                            $code
                        }
                    }
                }
            }
        }
    };
}

pub fn update(grid: &mut Grid) {
    let mut cells = HashSet::new();

    for y in 0..grid.height as isize {
        for x in 0..grid.width as isize {
            if let Some(cell) = grid.get_mut(x, y) {
                cell.updated = false;
                cells.insert(cell.id);
            }
        }
    }

    macro_rules! subtick {
        ($($cell:ident),*: $fn_name:ident) => {
            if $(cells.contains(&$cell))||* { $fn_name(grid); }
        }
    }

    subtick!(MIRROR          : do_mirrors);
    subtick!(CROSSMIRROR     : do_crossmirrors);
    subtick!(SUCKER          : do_suckers);
    subtick!(GENERATOR       : do_gens);
    subtick!(GENERATOR_CW, GENERATOR_CCW: do_angled_gens);
    subtick!(PHYSICAL_GENERATOR: do_physical_gens);
    subtick!(GENERATOR_CROSS : do_cross_gens);
    subtick!(REPLICATOR      : do_replicators);
    subtick!(POSTOFFICE      : do_postoffices);
    subtick!(ROTATOR_CW, ROTATOR_CCW: do_rotators);
    subtick!(ORIENTATOR      : do_orientators);
    subtick!(STONE           : do_stones);
    subtick!(MAILBOX         : do_mailboxes);
    subtick!(PULLSHER        : do_pullshers);
    subtick!(TRASHPULLER     : do_trashpullers);
    subtick!(PULLER          : do_pullers);
    subtick!(TRASHMOVER      : do_trashmovers);
    subtick!(MOVER           : do_movers);
    subtick!(SPEED           : do_speeds);
}

fn do_mirrors(grid: &mut Grid) {
    loop_each!(for x, y, cell in grid; {
        if cell.id == MIRROR && cell.direction.shrink(2) == Direction::Right && !cell.updated {
            cell.updated = true;
            let cell_left = grid.get_mut(x - 1, y);
            let cell_right = grid.get_mut(x + 1, y);
            if let Some(cell) = cell_left { if !can_move(cell, Direction::Right, MoveForce::Mirror) { continue; } }
            if let Some(cell) = cell_right { if !can_move(cell, Direction::Left, MoveForce::Mirror) { continue; } }

            let cell_left = cell_left.take();
            grid.set_cell(x - 1, y, cell_right.take());
            grid.set_cell(x + 1, y, cell_left);
        }
    });
    loop_each!(for x, y, cell in grid; {
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

fn do_crossmirrors(grid: &mut Grid) {
    loop_each!(for x, y, cell in grid; {
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

            let cell_up = grid.get_mut(x, y + 1);
            let cell_down = grid.get_mut(x, y - 1);
            let up_movable = if let Some(cell) = cell_up {
                can_move(cell, Direction::Down, MoveForce::Mirror)
            } else { true };
            let down_movable = if let Some(cell) = cell_down {
                can_move(cell, Direction::Up, MoveForce::Mirror)
            } else { true };
            if up_movable && down_movable {
                let cell_up = cell_left.take();
                grid.set_cell(x, y + 1, cell_down.take());
                grid.set_cell(x, y - 1, cell_up);
            }
        }
    });
}

fn do_suckers(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let push_offset = dir.to_vector();
    }, x, y, cell in grid; {
        if cell.id == SUCKER && cell.direction == dir && !cell.updated {
            cell.updated = true;
            pull(grid, x + push_offset.x, y + push_offset.y, dir.flip());
        }
    });
}

fn do_gens(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let push_offset = dir.to_vector();
        let cell_offset = dir.flip().to_vector();
    }, x, y, cell in grid; {
        if cell.id == GENERATOR && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(cell) = grid.get(x + cell_offset.x, y + cell_offset.y) {
                if can_generate(cell) {
                    push(grid, x + push_offset.x, y + push_offset.y, dir, 1, Some(cell.copy()));
                }
            }
        }
    });
}

fn do_angled_gens(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let cell_offset = dir.flip().to_vector();
    }, x, y, cell in grid; {
        if cell.id == GENERATOR_CW && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(cell) = grid.get(x + cell_offset.x, y + cell_offset.y) {
                let mut cell = cell.copy();
                cell.direction = cell.direction.rotate_right();
                let push_offset = dir.rotate_right().to_vector();
                push(grid, x + push_offset.x, y + push_offset.y, dir.rotate_right(), 1, Some(cell));
            }
        }
        else if cell.id == GENERATOR_CCW && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(cell) = grid.get(x + cell_offset.x, y + cell_offset.y) {
                let mut cell = cell.copy();
                cell.direction = cell.direction.rotate_left();
                let push_offset = dir.rotate_left().to_vector();
                push(grid, x + push_offset.x, y + push_offset.y, dir.rotate_left(), 1, Some(cell));
            }
        }
    });
}

fn do_physical_gens(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let push_offset = dir.to_vector();
        let cell_offset = dir.flip().to_vector();
    }, x, y, cell in grid; {
        if cell.id == PHYSICAL_GENERATOR && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(cell) = grid.get(x + cell_offset.x, y + cell_offset.y) {
                if can_generate(cell) && !push(grid, x + push_offset.x, y + push_offset.y, dir, 1, Some(cell.copy())).did_move() {
                    push(grid, x, y, dir.flip(), 1, Some(cell.copy()));
                }
            }
        }
    });
}

fn do_cross_gens(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let push_offset_1 = dir.to_vector();
        let cell_offset_1 = dir.flip().to_vector();
        let push_offset_2 = dir.rotate_left().to_vector();
        let cell_offset_2 = dir.rotate_right().to_vector();
    }, x, y, cell in grid; {
        if cell.id == GENERATOR_CROSS && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(cell) = grid.get(x + cell_offset_1.x, y + cell_offset_1.y) {
                if can_generate(cell) {
                    push(grid, x + push_offset_1.x, y + push_offset_1.y, dir, 1, Some(cell.copy()));
                }
            }
            if let Some(cell) = grid.get(x + cell_offset_2.x, y + cell_offset_2.y) {
                if can_generate(cell) {
                    push(grid, x + push_offset_2.x, y + push_offset_2.y, dir.rotate_left(), 1, Some(cell.copy()));
                }
            }
        }
    });
}

fn do_replicators(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let push_offset = dir.to_vector();
    }, x, y, cell in grid; {
        if cell.id == REPLICATOR && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(cell) = grid.get(x + push_offset.x, y + push_offset.y) {
                if can_generate(cell) {
                    push(grid, x + push_offset.x, y + push_offset.y, dir, 1, Some(cell.copy()));
                }
            }
        }
    });
}

fn do_postoffices(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let mail_offset = dir.flip().to_vector();
        let mailbox_offset = dir.to_vector();
    }, x, y, cell in grid; {
        if cell.id == POSTOFFICE && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(mailbox) = grid.get_mut(x + mailbox_offset.x, y + mailbox_offset.y) {
                if mailbox.id == MAILBOX {
                    if let Some(mail) = grid.get_mut(x + mail_offset.x, y + mail_offset.y) {
                        if can_move(mail, dir, MoveForce::Pull) {
                            mailbox.contained_cell = Some((mail.id, mail.direction - dir));
                            grid.delete(x + mail_offset.x, y + mail_offset.y);
                        }
                    }
                }
            }
        }
    });
}

fn do_rotators(grid: &mut Grid) {
    loop_each!(for x, y, cell in grid; {
        if !cell.updated {
            if cell.id == ROTATOR_CW {
                cell.updated = true;
                rotate_by(grid, x + 1, y, Direction::Down, Direction::Left);
                rotate_by(grid, x, y - 1, Direction::Down, Direction::Up);
                rotate_by(grid, x - 1, y, Direction::Down, Direction::Right);
                rotate_by(grid, x, y + 1, Direction::Down, Direction::Down);
            }
            else if cell.id == ROTATOR_CCW {
                cell.updated = true;
                rotate_by(grid, x + 1, y, Direction::Up, Direction::Left);
                rotate_by(grid, x, y - 1, Direction::Up, Direction::Up);
                rotate_by(grid, x - 1, y, Direction::Up, Direction::Right);
                rotate_by(grid, x, y + 1, Direction::Up, Direction::Down);
            }
        }
    });
}

fn do_orientators(grid: &mut Grid) {
    loop_each!(for x, y, cell in grid; {
        if cell.id == ORIENTATOR && !cell.updated {
            cell.updated = true;
            rotate_to(grid, x + 1, y, cell.direction, Direction::Left);
            rotate_to(grid, x, y - 1, cell.direction, Direction::Up);
            rotate_to(grid, x - 1, y, cell.direction, Direction::Right);
            rotate_to(grid, x, y + 1, cell.direction, Direction::Down);
        }
    });
}

fn do_stones(grid: &mut Grid) {
    loop_each_dir!(for dir, x, y, cell in grid; {
        if cell.id == STONE && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if push(grid, x, y, dir.rotate_right(), 1, None).did_move_survive() {
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
                    if push(grid, x, y, dir, 1, None).did_move_survive() {
                        push(grid, x + off.x, y + off.y, down, 1, None);
                    }
                }
                else if can_move_left && !can_move_right {
                    if push(grid, x, y, dir.flip(), 1, None).did_move_survive() {
                        push(grid, x, y, down, 1, None);
                    }
                }
                else if push(grid, x, y, dir, 1, None).did_move_survive() {
                    push(grid, x, y, down, 1, None);
                }
            }
        }
    });
}

fn do_mailboxes(grid: &mut Grid) {
    loop_each_dir!(for dir, x, y, cell in grid; {
        if cell.id == MAILBOX && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(contained) = cell.contained_cell {
                if !push(grid, x, y, dir, 1, None).did_move() {
                    grid.set(x, y, Cell::new(contained.0, dir + contained.1));
                }
            }
        }
    });
}

fn do_pullshers(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let off = dir.to_vector();
    }, x, y, cell in grid; {
        if cell.id == PULLSHER && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if push(grid, x, y, dir, 1, None).did_move() {
                pull(grid, x - off.x, y - off.y, dir);
            }
        }
    });
}

fn do_trashpullers(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let off = dir.flip().to_vector();
    }, x, y, cell in grid; {
        if cell.id == TRASHPULLER && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(pushed) = grid.get(x + off.x, y + off.y) {
                if can_move(pushed, dir, MoveForce::Pull) && !is_trash(pushed, dir) {
                    grid.delete(x + off.x, y + off.y);
                    if grid.get(x - off.x, y - off.y).is_none() {
                        pull(grid, x, y, dir);
                    }
                }
            }
        }
    });
}

fn do_pullers(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let off = dir.to_vector();
    }, x, y, cell in grid; {
        if cell.id == PULLER && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if grid.get(x + off.x, y + off.y).is_none() {
                pull(grid, x, y, dir);
            }
        }
    });
}

fn do_trashmovers(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let off = dir.to_vector();
    }, x, y, cell in grid; {
        if cell.id == TRASHMOVER && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if let Some(pushed) = grid.get(x + off.x, y + off.y) {
                if !can_move(pushed, dir, MoveForce::Mover) || is_trash(cell, dir) {
                    return;
                }
            }
            grid.delete(x + off.x, y + off.y);
            push(grid, x, y, dir, 0, None);
        }
    });
}

fn do_movers(grid: &mut Grid) {
    loop_each_dir!(for dir, x, y, cell in grid; {
        if cell.id == MOVER && cell.direction == dir && !cell.updated {
            cell.updated = true;
            push(grid, x, y, dir, 0, None);
        }
    });
}

fn do_speeds(grid: &mut Grid) {
    loop_each_dir!(for dir {
        let off = dir.to_vector();
    }, x, y, cell in grid; {
        if cell.id == SPEED && cell.direction == dir && !cell.updated {
            cell.updated = true;
            if grid.get(x + off.x, y + off.y).is_none() {
                push(grid, x, y, dir, 0, None);
            }
        }
    });
}
