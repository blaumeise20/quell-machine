use base64::{Engine, engine::general_purpose::STANDARD as base64};
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
            cell_arr.push(format!("{}{}", encode_num_62(cell.id()), u8::from(cell.direction())));
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
        let val = if let Some(cell) = cell { 1 + 4 * cell.id() as usize + usize::from(cell.direction()) }
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

    let mut cell_result = Vec::new();
    for (mut c, count) in cell_grouped {
        cell_result.append(&mut c);
        if count > 1 {
            cell_result.push(0xff);
            cell_result.append(&mut encode_num_s64(count));
        }
    }

    let mut compressor = Compressor::new(CompressionLvl::new(12).unwrap());
    let mut data = vec![0; compressor.zlib_compress_bound(cell_result.len())];
    let len = compressor.zlib_compress(&cell_result, &mut data).unwrap();

    result.push_str(&base64.encode(&data[..len]));
    result
}

pub fn import(input: &str) -> Result<Grid, &'static str> {
    let mut input = input.trim().split(';');

    let ty = input.next().ok_or("missing type specifier")?;
    match ty {
        "Q1" => decode_q1(decode_num_62(input.next().ok_or("missing width")?.chars()), decode_num_62(input.next().ok_or("missing height")?.chars()), input),
        "Q2" => decode_q2(decode_num_62(input.next().ok_or("missing width")?.chars()), decode_num_62(input.next().ok_or("missing height")?.chars()), input.next().ok_or("missing cell data")?),
        "V3" => decode_v3(decode_num_74(input.next().ok_or("missing width")?.chars()), decode_num_74(input.next().ok_or("missing height")?.chars()), input.next().ok_or("missing cell data")?),
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
            let id = decode_num_62(chars.into_iter()) as u8;
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

    let data = base64.decode(input).map_err(|_| "invalid base64")?;
    let mut decompressor = Decompressor::new();
    let mut uncompressed = vec![0; width * height * 3];
    let len = decompressor.zlib_decompress(&data, &mut uncompressed).unwrap();
    let mut input = uncompressed[..len].to_vec();

    let mut cell_groups = Vec::new();
    while let Some(val) = read_num_s64(&mut input) {
        if !input.is_empty() && input[0] == 0xff {
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
            grid.set(x, y, Cell::new((cell / 4) as u8, (cell % 4).into()));
        }
        x += 1;
        if x >= grid.width as isize {
            x = 0;
            y += 1;
        }
    }

    Ok(grid)
}

fn decode_v3(width: usize, height: usize, cells: &str) -> Result<Grid, &'static str> {
    let mut grid = Grid::new(width, height);

    let mut cell_index = 0;
    let mut cell_array = vec![];
    let mut cells = cells.chars();
    while let Some(mut ch) = cells.next() {
        if ch == ')' || ch == '(' {
            let offset: usize;
            let repeating_length: usize;

            // c = cells
            // o = offset (cells length - 1)
            // l = repeating length (cells length * (pattern count - 1))
            // c)ol
            // c(o)l
            // c(o(l)

            if ch == ')' {
                offset = decode_ch_74(cells.next().unwrap());
                repeating_length = decode_ch_74(cells.next().unwrap());
            }
            else {
                ch = cells.next().unwrap();
                // ch == '('

                let mut str = String::new();
                while ch != ')' && ch != '(' { str += &ch.to_string(); ch = cells.next().unwrap(); }
                offset = decode_num_74(str.chars());

                if ch == ')' {
                    repeating_length = decode_ch_74(cells.next().unwrap());
                }
                else {
                    ch = cells.next().unwrap();
                    let mut str2 = String::new();
                    while ch != ')' { str2 += &ch.to_string(); ch = cells.next().unwrap(); }
                    repeating_length = decode_num_74(str2.chars());
                }
            }

            for _ in 0..repeating_length {
                set_cell(&mut grid, cell_array[cell_index - offset - 1], cell_index).unwrap();
                cell_array.push(cell_array[cell_index - offset - 1]);
                cell_index += 1;
            }
        }
        else {
            set_cell(&mut grid, decode_ch_74(ch), cell_index).unwrap();
            cell_array.push(decode_ch_74(ch));
            cell_index += 1;
        }

        fn set_cell(grid: &mut Grid, cell: usize, index: usize) -> Option<()> {
            if cell < 72 {
                let cell_type = match (cell / 2) % 9 {
                    0 => crate::game::cell_data::GENERATOR,
                    1 => crate::game::cell_data::ROTATOR_CW,
                    2 => crate::game::cell_data::ROTATOR_CCW,
                    3 => crate::game::cell_data::MOVER,
                    4 => crate::game::cell_data::SLIDE,
                    5 => crate::game::cell_data::PUSH,
                    6 => crate::game::cell_data::WALL,
                    7 => crate::game::cell_data::ENEMY,
                    8 => crate::game::cell_data::TRASH,
                    _ => panic!("invalid cell type"),
                };

                let cell_dir = cell / 18;
                if cell_dir > 3 {
                    panic!("invalid cell direction");
                }

                if !grid.try_set(index, Some(Cell::new(cell_type, cell_dir.into()))) {
                    return None;
                }
            }
            Some(())
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

const NUMBER_KEY_74: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!$%&+-.=?^{}";

fn decode_num_74(chars: impl Iterator<Item = char>) -> usize {
    chars.fold(0, |acc, c| acc * 74 + NUMBER_KEY_74.find(c).unwrap())
}
fn decode_ch_74(ch: char) -> usize {
    NUMBER_KEY_74.find(ch).unwrap()
}

const NUMBER_KEY_S64: &[u8] = &[0x0,0x1,0x2,0x3,0x4,0x5,0x6,0x7,0x8,0x9,0xa,0xb,0xc,0xd,0xe,0xf,0x10,0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,0x19,0x1a,0x1b,0x1c,0x1d,0x1e,0x1f,0x20,0x21,0x22,0x23,0x24,0x25,0x26,0x27,0x28,0x29,0x2a,0x2b,0x2c,0x2d,0x2e,0x2f,0x30,0x31,0x32,0x33,0x34,0x35,0x36,0x37,0x38,0x39,0x3a,0x3b,0x3c,0x3d,0x3e,0x3f,0x40,0x41,0x42,0x43,0x44,0x45,0x46,0x47,0x48,0x49,0x4a,0x4b,0x4c,0x4d,0x4e,0x4f,0x50,0x51,0x52,0x53,0x54,0x55,0x56,0x57,0x58,0x59,0x5a,0x5b,0x5c,0x5d,0x5e,0x5f,0x60,0x61,0x62,0x63,0x64,0x65,0x66,0x67,0x68,0x69,0x6a,0x6b,0x6c,0x6d,0x6e,0x6f,0x70,0x71,0x72,0x73,0x74,0x75,0x76,0x77,0x78,0x79,0x7a,0x7b,0x7c,0x7d,0x7e,0x7f,0x80,0x81,0x82,0x83,0x84,0x85,0x86,0x87,0x88,0x89,0x8a,0x8b,0x8c,0x8d,0x8e,0x8f,0x90,0x91,0x92,0x93,0x94,0x95,0x96,0x97,0x98,0x99,0x9a,0x9b,0x9c,0x9d,0x9e,0x9f,0xa0,0xa1,0xa2,0xa3,0xa4,0xa5,0xa6,0xa7,0xa8,0xa9,0xaa,0xab,0xac,0xad,0xae,0xaf,0xb0,0xb1,0xb2,0xb3,0xb4,0xb5,0xb6,0xb7,0xb8,0xb9,0xba,0xbb,0xbc,0xbd,0xbe,0xbf,0xc0,0xc1,0xc2,0xc3,0xc4,0xc5,0xc6,0xc7,0xc8,0xc9,0xca,0xcb,0xcc,0xcd,0xce,0xcf,0xd0,0xd1,0xd2,0xd3,0xd4,0xd5,0xd6,0xd7,0xd8,0xd9,0xda,0xdb,0xdc,0xdd,0xde,0xdf,0xe0,0xe1,0xe2,0xe3,0xe4,0xe5,0xe6,0xe7,0xe8,0xe9,0xea,0xeb,0xec,0xed,0xee,0xef,0xf0,0xf1,0xf2,0xf3,0xf4,0xf5,0xf6,0xf7,0xf8,0xf9,0xfa];
const NUMBER_KEY_S64_LEN: usize = NUMBER_KEY_S64.len();
const NUMBER_KEY_S64_SPCHAR_LEN: usize = 4;

fn read_num_s64(buf: &mut Vec<u8>) -> Option<usize> {
    let mut num = 0;
    loop {
        if buf.is_empty() { return None; }
        let ch = buf.remove(0);

        if ch == 0xfb { num = num * NUMBER_KEY_S64_SPCHAR_LEN + 1; }
        else if ch == 0xfc { num = num * NUMBER_KEY_S64_SPCHAR_LEN + 2; }
        else if ch == 0xfd { num = num * NUMBER_KEY_S64_SPCHAR_LEN + 3; }
        else if ch == 0xfe { num = num * NUMBER_KEY_S64_SPCHAR_LEN + 4; }
        else { return Some(num * NUMBER_KEY_S64_LEN + NUMBER_KEY_S64.iter().position(|&c| c == ch).unwrap()); }
    }
}

fn encode_num_s64(num: usize) -> Vec<u8> {
    let mut changing = num / NUMBER_KEY_S64_LEN;
    let mut res = Vec::new();
    while changing > 0 {
             if changing >= 4 && (changing - 4) % NUMBER_KEY_S64_SPCHAR_LEN == 0 { changing = (changing - 4) / NUMBER_KEY_S64_SPCHAR_LEN; res.push(0xfe); }
        else if changing >= 3 && (changing - 3) % NUMBER_KEY_S64_SPCHAR_LEN == 0 { changing = (changing - 3) / NUMBER_KEY_S64_SPCHAR_LEN; res.push(0xfd); }
        else if changing >= 2 && (changing - 2) % NUMBER_KEY_S64_SPCHAR_LEN == 0 { changing = (changing - 2) / NUMBER_KEY_S64_SPCHAR_LEN; res.push(0xfc); }
        else if changing >= 1 && (changing - 1) % NUMBER_KEY_S64_SPCHAR_LEN == 0 { changing = (changing - 1) / NUMBER_KEY_S64_SPCHAR_LEN; res.push(0xfb); }
    }
    let mut res = res.into_iter().rev().collect::<Vec<u8>>();
    res.push(*NUMBER_KEY_S64.get(num % NUMBER_KEY_S64_LEN).unwrap());
    res
}

struct ChIter(Option<char>);

impl Iterator for ChIter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}
