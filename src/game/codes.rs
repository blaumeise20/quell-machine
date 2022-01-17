use std::str::from_utf8;

use libdeflater::{Compressor, CompressionLvl, Decompressor};

use super::cells::{Cell, Grid};

pub fn export_q1(grid: &Grid) -> String {
    let mut result = String::new();

    result.push_str("Q1;");
    result.push_str(encode_num_62(grid.width).as_str());
    result.push(';');
    result.push_str(encode_num_62(grid.height).as_str());
    result.push(';');

    let mut cell_arr = Vec::new();
    grid.for_each(|_, _, cell| {
        if let Some(cell) = cell {
            cell_arr.push(format!("{}{}", encode_num_62(cell.id), u8::from(cell.direction)));
        }
        else {
            cell_arr.push(String::new());
        }
    });

    let mut cell_grouped = Vec::new();
    let mut c_iter = cell_arr.into_iter();
    cell_grouped.push((c_iter.next().unwrap(), 1usize));
    for c in c_iter {
        let last = cell_grouped.last_mut().unwrap();
        if c == last.0 {
            last.1 += 1;
        }
        else {
            cell_grouped.push((c, 1));
        }
    }

    let mut cell_string = String::new();
    for (i, (c, count)) in cell_grouped.iter().enumerate() {
        cell_string.push_str(c.as_str());
        if *count > 1 {
            cell_string.push('+');
            cell_string.push_str(encode_num_62(*count).as_str());
        }
        if i < cell_grouped.len() - 1 {
            cell_string.push(';');
        }
    }

    result.push_str(&cell_string);
    result
}

pub fn export_q2(grid: &Grid) -> String {
    let mut result = String::new();

    result.push_str("Q2;");
    result.push_str(encode_num_62(grid.width).as_str());
    result.push(';');
    result.push_str(encode_num_62(grid.height).as_str());
    result.push(';');

    let mut cell_arr = Vec::new();
    grid.for_each(|_, _, cell| {
        let val = if let Some(cell) = cell { 1 + 4 * cell.id as usize + usize::from(cell.direction) }
                    else { 0 };
        cell_arr.push(encode_num_s64(val));
    });

    let mut cell_grouped = Vec::new();
    let mut c_iter = cell_arr.into_iter();
    cell_grouped.push((c_iter.next().unwrap(), 1usize));
    for c in c_iter {
        let last = cell_grouped.last_mut().unwrap();
        if c == last.0 {
            last.1 += 1;
        }
        else {
            cell_grouped.push((c, 1));
        }
    }

    let mut cell_result = String::new();
    for (c, count) in cell_grouped {
        cell_result.push_str(c.as_str());
        if count > 1 {
            cell_result.push('*');
            cell_result.push_str(&encode_num_s64(count));
        }
    }

    let mut compressor = Compressor::new(CompressionLvl::new(12).unwrap());
    let mut data = vec![0; compressor.zlib_compress_bound(cell_result.len())];
    let len = compressor.zlib_compress(cell_result.as_bytes(), &mut data).unwrap();

    result.push_str(&base64::encode(&data[..len]));
    result
}

pub fn import(input: &str) -> Result<Grid, &'static str> {
    let mut input = input.trim().split(';');

    let ty = input.next().ok_or("missing type specifier")?;
    let width = decode_num_62(input.next().ok_or("missing width")?.chars());
    let height = decode_num_62(input.next().ok_or("missing height")?.chars());

    match ty {
        "Q1" => decode_q1(width, height, input),
        "Q2" => decode_q2(width, height, input.next().ok_or("missing cell data")?),
        _ => Err("unknown code type"),
    }
}

fn decode_q1<'a>(width: usize, height: usize, input: impl Iterator<Item = &'a str>) -> Result<Grid, &'static str> {
    let mut grid = Grid::new(width, height);

    let mut cell_groups = Vec::new();
    for cell_group in input {
        if let Some(pos) = cell_group.find('+') {
            cell_groups.push((&cell_group[..pos], decode_num_62(cell_group[pos+1..].chars())));
        }
        else {
            cell_groups.push((cell_group, 1));
        }
    }

    let mut cell_arr = Vec::new();
    for (cell, mut count) in cell_groups {
        while count > 0 {
            cell_arr.push(cell);
            count -= 1;
        }
    }
    let cell_arr = cell_arr.into_iter();

    let mut x = 0;
    let mut y = 0;
    for cell_str in cell_arr {
        if !cell_str.is_empty() {
            let mut chars = cell_str.chars().collect::<Vec<_>>();
            let direction = chars.pop().unwrap().to_digit(10).unwrap() as u8;
            let id = decode_num_62(chars.into_iter()) as u16;
            grid.set(x, y, Cell::new(id, direction.into()));
        }
        x += 1;
        if x >= grid.width as isize {
            x = 0;
            y += 1;
        }
    }

    Ok(grid)
}

fn decode_q2(width: usize, height: usize, input: &str) -> Result<Grid, &'static str> {
    let mut grid = Grid::new(width, height);

    let data = base64::decode(input).map_err(|_| "invalid base64")?;
    let mut decompressor = Decompressor::new();
    let mut uncompressed = vec![0; width * height * 3];
    let len = decompressor.zlib_decompress(&data, &mut uncompressed).unwrap();
    let mut input = from_utf8(&uncompressed[..len]).expect("should never happen").to_string();

    let mut cell_groups = Vec::new();
    while let Some(val) = read_num_s64(&mut input) {
        if !input.is_empty() && input.as_bytes()[0] == b'*' {
            input.remove(0);
            cell_groups.push((val, read_num_s64(&mut input).unwrap()));
        }
        else {
            cell_groups.push((val, 1));
        }
    }

    let mut cell_arr = Vec::new();
    for (cell, mut count) in cell_groups {
        while count > 0 {
            cell_arr.push(cell);
            count -= 1;
        }
    }
    let cell_arr = cell_arr.into_iter();

    let mut x = 0;
    let mut y = 0;
    for cell in cell_arr {
        if cell != 0 {
            let cell = cell - 1;
            grid.set(x, y, Cell::new((cell / 4) as u16, (cell % 4).into()));
        }
        x += 1;
        if x >= grid.width as isize {
            x = 0;
            y += 1;
        }
    }

    Ok(grid)
}

const NUMBER_KEY_62: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

fn encode_num_62<N: Into<usize>>(num: N) -> String {
    let mut num = num.into();
    let mut result = String::new();
    while num > 0 {
        result.push(NUMBER_KEY_62.chars().nth(num % 62).unwrap());
        num /= 62;
    }
    result.chars().rev().collect()
}

fn decode_num_62(chars: impl Iterator<Item = char>) -> usize {
    chars.fold(0, |acc, c| acc * 62 + NUMBER_KEY_62.find(c).unwrap())
}

const NUMBER_KEY_S64: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUV+/";
const NUMBER_KEY_S64_LEN: usize = NUMBER_KEY_S64.len();
const NUMBER_KEY_S64_SPCHAR_LEN: usize = 4;

fn read_num_s64(string: &mut String) -> Option<usize> {
    let mut num = 0;
    loop {
        if string.is_empty() { return None; }
        let ch = string.remove(0);

        if ch == 'W' { num = num * NUMBER_KEY_S64_SPCHAR_LEN + 1; }
        else if ch == 'X' { num = num * NUMBER_KEY_S64_SPCHAR_LEN + 2; }
        else if ch == 'Y' { num = num * NUMBER_KEY_S64_SPCHAR_LEN + 3; }
        else if ch == 'Z' { num = num * NUMBER_KEY_S64_SPCHAR_LEN + 4; }
        else { return Some(num * NUMBER_KEY_S64_LEN + NUMBER_KEY_S64.find(ch).unwrap()); }
    }
}

fn encode_num_s64(num: usize) -> String {
    let mut changing = num / NUMBER_KEY_S64_LEN;
    let mut res = String::new();
    while changing > 0 {
             if changing >= 4 && (changing - 4) % NUMBER_KEY_S64_SPCHAR_LEN == 0 { changing = (changing - 4) / NUMBER_KEY_S64_SPCHAR_LEN; res.push('Z'); }
        else if changing >= 3 && (changing - 3) % NUMBER_KEY_S64_SPCHAR_LEN == 0 { changing = (changing - 3) / NUMBER_KEY_S64_SPCHAR_LEN; res.push('Y'); }
        else if changing >= 2 && (changing - 2) % NUMBER_KEY_S64_SPCHAR_LEN == 0 { changing = (changing - 2) / NUMBER_KEY_S64_SPCHAR_LEN; res.push('X'); }
        else if changing >= 1 && (changing - 1) % NUMBER_KEY_S64_SPCHAR_LEN == 0 { changing = (changing - 1) / NUMBER_KEY_S64_SPCHAR_LEN; res.push('W'); }
    }
    res.chars().rev().collect::<String>() + &NUMBER_KEY_S64.chars().nth(num % NUMBER_KEY_S64_LEN).unwrap().to_string()
}
