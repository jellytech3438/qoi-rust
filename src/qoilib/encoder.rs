use colored::*;
use image::EncodableLayout;
use std::{env::current_dir, fmt::Error, io::Write};

use super::{
    header::{
        qoi_channels, qoi_header, QOI_END, QOI_OP_DIFF, QOI_OP_INDEX, QOI_OP_LUMA, QOI_OP_RGB,
        QOI_OP_RUN,
    },
    PixelHashMap, Pixels,
};

pub struct Encoder<'a> {
    // condier [u8; 4] as Rgb<u8>
    // then data is array of Rgb
    data: &'a [[u8; 4]],
    header: qoi_header,
}

impl<'a> Encoder<'a> {
    pub fn new(
        data: &'a [[u8; 4]],
        width: u32,
        height: u32,
        channels: qoi_channels,
        colorspace: u8,
    ) -> Self {
        Encoder {
            data,
            header: qoi_header::new(width, height, channels, colorspace),
        }
    }

    // return write bytes
    pub fn encode_to_buffer<W>(
        &self,
        buffer: &mut std::io::BufWriter<W>,
    ) -> Result<usize, std::fmt::Error>
    where
        W: Write,
    {
        let pxs_write = self.header.width * self.header.height;
        let mut prevpx = Pixels::start_prev();
        let mut hashmap = PixelHashMap::new();

        let mut prevpxindex = prevpx.hash();

        // write header into buffer
        buffer
            .write(&self.header.to_bytes())
            .expect("write header error");

        let mut run = 0;
        let mut cnt = 0;
        let mut px = Pixels::start_prev();

        while cnt < pxs_write {
            // get next px
            px = Pixels::from(self.data[cnt as usize]);
            let mut c = format!("{}", cnt).on_custom_color(CustomColor::new(px.r, px.g, px.b));
            println!("{}", c);

            cnt += 1;

            // run != 63 and 64
            if px == prevpx {
                run += 1;
                if run == 62 {
                    buffer.write(((QOI_OP_RUN << 6) | run).to_ne_bytes().as_ref());
                    run = 0;
                }
            } else {
                let mut new_run = false;
                if run != 0 {
                    new_run = true;
                    if run == 1 {
                        // run only count once which mean the prev px only find sequentialy one time
                        buffer.write(((QOI_OP_INDEX << 6) | prevpx.hash()).to_ne_bytes().as_ref());
                    } else {
                        // find a new none sequence px so write run into buffer first
                        buffer.write(((QOI_OP_RUN << 6) | run).to_ne_bytes().as_ref());
                        buffer.write(((QOI_OP_INDEX << 6) | prevpx.hash()).to_ne_bytes().as_ref());
                    }
                    run = 0;
                }

                // get previous index
                prevpxindex = prevpx.hash();

                if px == hashmap[prevpxindex] {
                    buffer.write(((QOI_OP_INDEX << 6) | prevpxindex).to_ne_bytes().as_ref());
                } else {
                    // calculate pixel different

                    let diff_rgb = -2..1;
                    let diff_rb = -8..7;
                    let diff_g = -32..31;

                    let dr = px.dr(prevpx);
                    let dg = px.dg(prevpx);
                    let db = px.db(prevpx);

                    if diff_rgb.contains(&dr) && diff_rgb.contains(&dg) && diff_rgb.contains(&db) {
                        let bytes_o_l = px.rgb_to_bytes(prevpx, super::DiffType::DIFF);
                        buffer.write(((QOI_OP_DIFF << 6) | bytes_o_l[1]).to_ne_bytes().as_ref());
                    } else if diff_g.contains(&dg) && diff_rb.contains(&dr) && diff_rb.contains(&db)
                    {
                        let bytes_o_l = px.rgb_to_bytes(prevpx, super::DiffType::LUMA);
                        buffer.write([(QOI_OP_LUMA << 6) | bytes_o_l[0], bytes_o_l[1]].as_ref());
                    } else {
                        buffer.write([(QOI_OP_RGB), (px.r), (px.g), (px.b)].as_ref());
                    }
                }
            }
            hashmap[px.hash()] = px;
            prevpx = px;
        }

        // last pixel is run and not write in
        if run != 0 {
            if run == 1 {
                // run only count once which mean the prev px only find sequentialy one time
                buffer.write(((QOI_OP_INDEX << 6) | prevpx.hash()).to_ne_bytes().as_ref());
            } else {
                // find a new none sequence px so write run into buffer first
                buffer.write(((QOI_OP_RUN << 6) | run).to_ne_bytes().as_ref());
            }
            run = 0;
        }

        buffer.write(QOI_END.to_ne_bytes().as_ref());

        return Ok(buffer.buffer().len());
    }
}
