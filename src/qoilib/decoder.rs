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

        let magic = read_u32(&mut self.reader);
        let width = read_u32(&mut self.reader).unwrap() as u32;
        let height = read_u32(&mut self.reader).unwrap() as u32;
        let header = qoi_header::new(width, height, qoi_channels::Rgb, 0);

        let pxs_write = width * height - 1;

        let mut cnt = 0;
        let mut run = 0;
        let mut rtn_data: Vec<[u8; 4]> = Vec::new();

        while cnt < pxs_write {
            println!("{}", cnt);
            cnt += 1;
            run -= 1;
            if run < 0 {
                let byte_zero = read_u8(&mut self.reader).unwrap();
                match byte_zero as u8 {
                    255 => {
                        // RGBA
                        let r = read_u8(&mut self.reader).unwrap() as u8;
                        let g = read_u8(&mut self.reader).unwrap() as u8;
                        let b = read_u8(&mut self.reader).unwrap() as u8;
                        let a = read_u8(&mut self.reader).unwrap() as u8;
                        rtn_data.push([r, g, b, a]);
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
                    }
                    192..=253 => {
                        // RUN
                        run = byte_zero - 192;
                    }
                    128..=191 => {
                        // LUMA
                        let dg = (byte_zero as u8) - 128;
                        let next = read_u8(&mut self.reader).unwrap();
                        let dr_dg = next >> 4;
                        let db_dg = next - dr_dg;

                        let r = prevpx.r + dr_dg + dg;
                        let g = prevpx.g + dg;
                        let b = prevpx.b + db_dg + dg;

                        rtn_data.push([r, g, b, 255]);
                        prevpx = Pixels::new(r, g, b, 255);
                        hashmap[prevpx.hash()] = prevpx;
                    }
                    64..=127 => {
                        // DIFF
                        let rgb = byte_zero - 64;
                        let dr = (rgb & 0b0000_1111) >> 4;
                        let dg = (rgb & 0b0011_0011) >> 2;
                        let db = rgb & 0b0011_1100;

                        let r = prevpx.r + dr;
                        let g = prevpx.g + dg;
                        let b = prevpx.b + db;

                        rtn_data.push([r, g, b, 255]);
                        prevpx = Pixels::new(r, g, b, 255);
                        hashmap[prevpx.hash()] = prevpx;
                    }
                    0..=63 => {
                        // INDEX
                        let index = byte_zero;
                        let px = hashmap[index];

                        rtn_data.push([px.r, px.g, px.b, px.a]);
                        prevpx = px;
                        hashmap[prevpx.hash()] = prevpx;
                    }
                }
            } else {
                rtn_data.push([prevpx.r, prevpx.g, prevpx.b, prevpx.a]);
            }
        }

        Ok((rtn_data, header))
    }
}
