use std::num::Wrapping;

use super::header::QOI_OP_RGB;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixels {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub enum DiffType {
    DIFF,
    LUMA,
}

pub struct PixelHashMap([Pixels; 64]);

impl Pixels {
    pub fn new(r: u8, g: u8, b: u8, _a: u8) -> Self {
        Pixels { r, g, b, a: _a }
    }
    pub fn start_prev() -> Self {
        Pixels {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn hash(&self) -> u8 {
        (self.r * 3 + self.g * 5 + self.b * 7 + self.a * 11) % 64
    }

    pub fn dr(&self, rhs: Pixels) -> i8 {
        self.r.wrapping_sub(rhs.r) as i8
    }
    pub fn dg(&self, rhs: Pixels) -> i8 {
        self.g.wrapping_sub(rhs.g) as i8
    }
    pub fn db(&self, rhs: Pixels) -> i8 {
        self.b.wrapping_sub(rhs.b) as i8
    }
    pub fn rgb_to_bytes(&self, prev: Pixels, diff_type: DiffType) -> [u8; 2] {
        match diff_type {
            DiffType::DIFF => {
                let dr = (self.dr(prev) + 2) as u8;
                let dg = (self.dg(prev) + 2) as u8;
                let db = (self.db(prev) + 2) as u8;
                [0, ((dr << 4) | (dg << 2) | (db))]
            }
            DiffType::LUMA => {
                let dr = (self.dr(prev)) as u8;
                let dg = (self.dg(prev)) as u8;
                let db = (self.db(prev)) as u8;
                let dr_dg = dr - (dg);
                let db_dg = db - (dg);
                [
                    ((dg + 32) & 0b0011_1111),
                    (((dr_dg + 8) << 4) | (db_dg + 8)),
                ]
            }
        }
    }
}

impl From<[u8; 4]> for Pixels {
    fn from(value: [u8; 4]) -> Self {
        Pixels {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }
    }
}

impl PixelHashMap {
    pub fn new() -> Self {
        PixelHashMap([Pixels::new(0, 0, 0, 0); 64])
    }
}

impl std::ops::Index<u8> for PixelHashMap {
    type Output = Pixels;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[usize::from(index)]
    }
}

impl std::ops::IndexMut<u8> for PixelHashMap {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
    }
}
