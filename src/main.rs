mod qoilib;

use std::io::{BufReader, BufWriter, Write};
use std::{fs::File, ops::Deref};

use image::{Pixel, Pixels as IMGPixels, Rgb, RgbImage, Rgba, RgbaImage};
use qoilib::decoder::{read_u32, read_u8, Decoder};
use qoilib::encoder::Encoder;

fn write_to_file() {
    let test = File::create("test.txt").unwrap();
    let mut buffer = BufWriter::new(test);
    for i in 0..10 as u8 {
        buffer.write(&[i]).unwrap();
    }
    // buffer.flush().unwrap();
}

fn read_from_file() {
    let test = File::open("test.txt").unwrap();
    let mut buffer = BufReader::new(test);
    for i in 0..10 {
        println!("{}", read_u8(&mut buffer).unwrap() as u32);
    }
}

fn main() {
    // write_to_file();
    // read_from_file();
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
    let pxs = decoder.decode().unwrap().0;
    let header = decoder.decode().unwrap().1;
    let img_pxs: Vec<Rgba<u8>> = pxs.iter().map(|p| Rgba(*p)).collect();

    let mut op_img = RgbaImage::new(header.width, header.height);
    for i in 0..op_img.width() {
        for j in 0..op_img.height() {
            op_img.put_pixel(i, j, img_pxs[(i * op_img.width() + j) as usize]);
        }
    }
    op_img.save("img_op.png").unwrap();
}
