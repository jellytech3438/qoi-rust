use colored::*;
use std::io::Read;
use std::mem::MaybeUninit;

use super::header::{qoi_channels, qoi_header};
use super::{PixelHashMap, Pixels};

// reference from: https://github.com/ChevyRay/qoi_rs/blob/457236d7e3a488d1751b175abfc6b448338898b1/src/decode.rs#L14

pub fn read<R: Read, const N: usize>(input: &mut R) -> Result<[u8; N], std::fmt::Error> {
    let mut bytes: [u8; N] = unsafe { MaybeUninit::uninit().assume_init() };
    input.read_exact(&mut bytes);
    Ok(bytes)
}

pub fn read_u8<R: Read>(input: &mut R) -> Result<u8, std::fmt::Error> {
    Ok(read::<R, 1>(input).unwrap()[0])
}

pub fn read_u32<R: Read>(input: &mut R) -> Result<u32, std::fmt::Error> {
    Ok(u32::from_be_bytes(read::<R, 4>(input).unwrap()))
}

pub struct Decoder<R: Read> {
    reader: R,
}

impl<R> Decoder<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Decoder { reader }
    }

    pub fn decode(&mut self) -> Result<(Vec<[u8; 4]>, qoi_header), std::fmt::Error> {
        let mut prevpx = Pixels::start_prev();
        let mut hashmap = PixelHashMap::new();

        let mut prevpxindex = prevpx.hash();

        println!();
        println!("DECODE START");
        let magic = read_u32(&mut self.reader).unwrap();
        // println!("magic: {:b}", magic);
        let width = read_u32(&mut self.reader).unwrap() as u32;
        // println!("w: {:b}", width);
        let height = read_u32(&mut self.reader).unwrap() as u32;
        // println!("h: {:b}", height);
        let channels = read_u8(&mut self.reader).unwrap() as u8;
        // println!("channels: {:b}", channels);
        let colorspace = read_u8(&mut self.reader).unwrap() as u8;
        // println!("colorspace: {:b}", colorspace);
        let header = qoi_header::new(width, height, qoi_channels::Rgb, colorspace);
        println!();

        let pxs_write = width * height;

        let mut cnt = 0;
        let mut run: i32 = 0;
        let mut new_run = false;
        let mut rtn_data: Vec<[u8; 4]> = Vec::new();

        while cnt < pxs_write {
            cnt += 1;
            run -= 1;
            if run < 0 {
                let byte_zero = read_u8(&mut self.reader).unwrap();
                // print!("{:b} ", byte_zero);
                match byte_zero as u8 {
                    255 => {
                        // RGBA
                        let r = read_u8(&mut self.reader).unwrap() as u8;
                        let g = read_u8(&mut self.reader).unwrap() as u8;
                        let b = read_u8(&mut self.reader).unwrap() as u8;
                        let a = read_u8(&mut self.reader).unwrap() as u8;
                        rtn_data.push([r, g, b, a]);
                        println!(
                            "{}",
                            format!("RGBA {}", cnt).on_custom_color(CustomColor::new(r, g, b)),
                        );
                        prevpx = Pixels::new(r, g, b, a);
                        hashmap[prevpx.hash()] = prevpx;
                    }
                    254 => {
                        // RGB
                        let r = read_u8(&mut self.reader).unwrap() as u8;
                        let g = read_u8(&mut self.reader).unwrap() as u8;
                        let b = read_u8(&mut self.reader).unwrap() as u8;
                        rtn_data.push([r, g, b, 255]);
                        prevpx = Pixels::new(r, g, b, 255);
                        hashmap[prevpx.hash()] = prevpx;
                        println!(
                            "{}",
                            format!("RGB {}", cnt).on_custom_color(CustomColor::new(r, g, b))
                        );
                    }
                    192..=253 => {
                        // RUN
                        run = (byte_zero - 192) as i32;
                        cnt -= 1;
                        new_run = true;
                    }
                    128..=191 => {
                        // LUMA
                        let dg = ((byte_zero & 0b0011_1111) as u8) - 32;
                        let next = read_u8(&mut self.reader).unwrap();
                        let dr_dg = (next & 0b1111_0000) >> 4;
                        let db_dg = next & 0b0000_1111;

                        let dr = dr_dg + dg - 8;
                        let db = db_dg + dg - 8;

                        let r = prevpx.r + dr;
                        let g = prevpx.g + dg;
                        let b = prevpx.b + db;

                        rtn_data.push([r, g, b, 255]);
                        prevpx = Pixels::new(r, g, b, 255);
                        hashmap[prevpx.hash()] = prevpx;
                        println!(
                            "{}",
                            format!("LUMA {}", cnt,).on_custom_color(CustomColor::new(r, g, b)),
                        );
                    }
                    64..=127 => {
                        // DIFF
                        let rgb = byte_zero - 64;
                        let dr = ((rgb & 0b0011_0000) >> 4) - 2;
                        let dg = ((rgb & 0b0000_1100) >> 2) - 2;
                        let db = (rgb & 0b0000_0011) - 2;

                        let r = prevpx.r + dr;
                        let g = prevpx.g + dg;
                        let b = prevpx.b + db;

                        rtn_data.push([r, g, b, 255]);
                        prevpx = Pixels::new(r, g, b, 255);
                        hashmap[prevpx.hash()] = prevpx;
                        println!(
                            "{}",
                            format!("DIFF {}", cnt).on_custom_color(CustomColor::new(r, g, b)),
                        );
                    }
                    0..=63 => {
                        // INDEX
                        let index = byte_zero;
                        let px = hashmap[index];

                        rtn_data.push([px.r, px.g, px.b, px.a]);
                        prevpx = px;
                        hashmap[prevpx.hash()] = prevpx;
                        println!(
                            "{}",
                            format!("INDEX {}", cnt)
                                .on_custom_color(CustomColor::new(px.r, px.g, px.b)),
                        );
                    }
                }
            } else {
                if new_run {
                    let byte_zero = read_u8(&mut self.reader).unwrap();
                    match byte_zero as u8 {
                        255 => {
                            // RGBA
                            let r = read_u8(&mut self.reader).unwrap() as u8;
                            let g = read_u8(&mut self.reader).unwrap() as u8;
                            let b = read_u8(&mut self.reader).unwrap() as u8;
                            let a = read_u8(&mut self.reader).unwrap() as u8;
                            prevpx = Pixels::new(r, g, b, a);
                            hashmap[prevpx.hash()] = prevpx;
                        }
                        254 => {
                            // RGB
                            let r = read_u8(&mut self.reader).unwrap() as u8;
                            let g = read_u8(&mut self.reader).unwrap() as u8;
                            let b = read_u8(&mut self.reader).unwrap() as u8;
                            prevpx = Pixels::new(r, g, b, 255);
                            hashmap[prevpx.hash()] = prevpx;
                        }
                        192..=253 => {
                            // RUN
                            // more than 62
                            run += (byte_zero - 192) as i32 + 1;
                            new_run = true;
                        }
                        128..=191 => {
                            // LUMA
                            let dg = (byte_zero as u8) - 128 - 32;
                            let next = read_u8(&mut self.reader).unwrap();
                            let dr_dg = (next >> 4) - 8;
                            let db_dg = (next & 0b0000_1111) - 8;

                            let dr = dr_dg + dg;
                            let db = db_dg + dg;

                            let r = prevpx.r + dr;
                            let g = prevpx.g + dg;
                            let b = prevpx.b + db;

                            prevpx = Pixels::new(r, g, b, 255);
                            hashmap[prevpx.hash()] = prevpx;
                        }
                        64..=127 => {
                            // DIFF
                            let rgb = byte_zero - 64;
                            let dr = ((rgb & 0b0011_0000) >> 4) - 2;
                            let dg = ((rgb & 0b0000_1100) >> 2) - 2;
                            let db = (rgb & 0b0000_0011) - 2;

                            let r = prevpx.r + dr;
                            let g = prevpx.g + dg;
                            let b = prevpx.b + db;

                            prevpx = Pixels::new(r, g, b, 255);
                            hashmap[prevpx.hash()] = prevpx;
                        }
                        0..=63 => {
                            // INDEX
                            let index = byte_zero;
                            let px = hashmap[index];

                            prevpx = px;
                            hashmap[prevpx.hash()] = prevpx;
                        }
                    }
                    new_run = false;
                }
                rtn_data.push([prevpx.r, prevpx.g, prevpx.b, prevpx.a]);
                println!(
                    "{}",
                    format!("RUN {}", cnt)
                        .on_custom_color(CustomColor::new(prevpx.r, prevpx.g, prevpx.b))
                );
            }
        }

        println!("length: {}", rtn_data.len());
        println!("{:?}", rtn_data);
        Ok((rtn_data, header))
    }
}
