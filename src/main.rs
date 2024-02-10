mod qoilib;

use std::io::{BufReader, BufWriter, Write};
use std::{fs::File, ops::Deref};

use image::{Pixel, Pixels as IMGPixels, Rgb, RgbImage, Rgba, RgbaImage};
use qoilib::decoder::{read_u32, read_u8, Decoder};
use qoilib::encoder::Encoder;

fn main() {
    test_encode();
    test_decode();
}

fn test_encode() {
    let img = image::open("img.png").unwrap();

    // test encode
    // image -> into_vec() -> make <P::SubPixel> to <[u8;4]>
    // use this as encoder's data
    let img_vec: Vec<[u8; 4]> = img
        .to_rgba8()
        .pixels()
        .map(|p| {
            [
                p.channels()[0],
                p.channels()[1],
                p.channels()[2],
                p.channels()[3],
            ]
        })
        .collect();
    let mut encoder = Encoder::new(
        img_vec.as_slice(),
        img.width(),
        img.height(),
        qoilib::header::qoi_channels::Rgb,
        0,
    );
    let mut op_file = File::create("img_op.qoi").unwrap();
    let mut buffer = BufWriter::new(op_file);

    // start encoding
    encoder.encode_to_buffer(&mut buffer);
}

fn test_decode() {
    let mut op_file = File::open("img_op.qoi").unwrap();
    let mut decoder = Decoder::new(op_file);

    // start decoding
    let (pxs, header) = decoder.decode().unwrap();
    let img_pxs: Vec<Rgba<u8>> = pxs.iter().map(|p| Rgba(*p)).collect();

    let mut op_img = RgbaImage::new(header.width, header.height);
    for i in 0..op_img.width() {
        for j in 0..op_img.height() {
            op_img.put_pixel(i, j, img_pxs[(j * op_img.width() + i) as usize]);
        }
    }
    op_img.save("img_op.png").unwrap();
}
