use super::cells::{grid, Cell, Grid};

pub fn export() -> String { unsafe {
    let mut result = String::new();

    result.push_str("Q1;");
    result.push_str(encode_num(grid.width).as_str());
    result.push(';');
    result.push_str(encode_num(grid.height).as_str());
    result.push(';');

    let mut cell_arr = Vec::new();
    grid.for_each(|_, _, cell| {
        if let Some(cell) = cell {
            cell_arr.push(encode_cell(cell));
        }
        else {
            cell_arr.push(String::new());
        }
    });

    let mut cell_grouped = Vec::new();
    let mut c_iter = cell_arr.into_iter();
    cell_grouped.push((c_iter.next().unwrap(), 1));
    for c in c_iter {
        let last = cell_grouped.last_mut().unwrap();
        if c == last.0 {
            last.1 += 1;
        }
        else {
            cell_grouped.push((c, 1));
        }
    }

    for (c, count) in cell_grouped {
        result.push_str(encode_cell_group(c, count).as_str());
        result.push(';');
    }

    result
} }

pub fn import(input: &str) -> Result<(), &'static str> { unsafe {
    let mut input = input.trim().split(';');

    let q1 = input.next().ok_or("missing Q1")?;
    if q1 != "Q1" { return Err("invalid Q1"); }

    let width = decode_num(input.next().ok_or("missing width")?.chars());
    let height = decode_num(input.next().ok_or("missing height")?.chars());

    grid = Grid::new(width, height);
    grid.init();

    let mut cell_groups = Vec::new();
    for cell_group in input {
        if cell_group.is_empty() { continue; }
        cell_groups.push(decode_cell_group(cell_group));
    }

    let mut cell_arr = Vec::new();
    for (cell, mut count) in cell_groups {
        while count > 0 {
            cell_arr.push(cell);
            count -= 1;
        }
    }
    let mut cell_arr = cell_arr.into_iter();

    for y in 0..grid.height {
        for x in 0..grid.width {
            if let Some(cell_str) = cell_arr.next() {
                if cell_str.is_empty() { continue; }
                grid.set(x as isize, y as isize, decode_cell(cell_str));
            }
        }
    }

    Ok(())
} }

fn encode_cell_group(cell: String, count: usize) -> String {
    let mut result = String::new();

    result.push_str(cell.as_str());
    if count > 1 {
        result.push('+');
        result.push_str(encode_num(count).as_str());
    }

    result
}

fn decode_cell_group(cg: &str) -> (&str, usize) {
    if let Some(pos) = cg.find('+') {
        (&cg[..pos], decode_num(cg[pos+1..].chars()))
    }
    else {
        (cg, 1)
    }
}

fn encode_cell(cell: &Cell) -> String {
    format!("{}{}", encode_num(cell.id), u8::from(cell.direction))
}

fn decode_cell(cell: &str) -> Cell {
    let mut chars = cell.chars().collect::<Vec<_>>();
    let direction = chars.pop().unwrap().to_digit(10).unwrap() as u8;
    let id = decode_num(chars.into_iter()) as u16;
    Cell::new(id, direction.into())
}

const NUMBER_KEY: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

fn encode_num<N: Into<usize>>(num: N) -> String {
    let mut num = num.into();
    let mut result = String::new();
    while num > 0 {
        result.push(NUMBER_KEY.chars().nth(num % 62).unwrap());
        num /= 62;
    }
    result.chars().rev().collect()
}

fn decode_num(chars: impl Iterator<Item = char>) -> usize {
    chars.fold(0, |acc, c| acc * 62 + NUMBER_KEY.find(c).unwrap())
}
