pub enum qoi_channels {
    Rgb = 3,
    Rgba = 4,
}

impl qoi_channels {
    pub fn to_bytes(&self) -> u8 {
        match self {
            Self::Rgb => 3,
            Self::Rgba => 4,
        }
    }
}

pub struct qoi_header {
    magic: [u8; 4],
    pub width: u32,
    pub height: u32,
    channels: qoi_channels,
    colorspace: u8,
}

impl qoi_header {
    pub fn new(width: u32, height: u32, channels: qoi_channels, colorspace: u8) -> Self {
        qoi_header {
            magic: QOI_MAGIC.to_owned(),
            width,
            height,
            channels,
            colorspace,
        }
    }
    pub fn to_bytes(&self) -> [u8; 14] {
        let mut header: [u8; 14] = [0; 14];

        for i in 0..4 {
            header[i] = self.magic[i];
        }

        let w = self.width();
        for i in 4..8 {
            header[i] = w[i - 4];
        }

        let h = self.height();
        for i in 8..12 {
            header[i] = h[i - 8];
        }

        header[12] = self.channels.to_bytes();
        header[13] = self.colorspace;

        header
    }
    pub fn width(&self) -> [u8; 4] {
        let byte1: u8 = ((self.width >> 24) & 0xff) as u8;
        let byte2: u8 = ((self.width >> 16) & 0xff) as u8;
        let byte3: u8 = ((self.width >> 8) & 0xff) as u8;
        let byte4: u8 = ((self.width) & 0xff) as u8;
        [byte1, byte2, byte3, byte4]
    }
    pub fn height(&self) -> [u8; 4] {
        let byte1: u8 = ((self.height >> 24) & 0xff) as u8;
        let byte2: u8 = ((self.height >> 16) & 0xff) as u8;
        let byte3: u8 = ((self.height >> 8) & 0xff) as u8;
        let byte4: u8 = ((self.height) & 0xff) as u8;
        [byte1, byte2, byte3, byte4]
    }
}

pub(crate) const QOI_MAGIC: &[u8; 4] = b"qoif";
// byte 0
// if compare px and prevpx are so large that bytes compression is not enought
// we directly store the full color information
// (the most inefficient way)
pub(crate) const QOI_OP_RGB: u8 = 254;
pub(crate) const QOI_OP_RGBA: u8 = 255;

// (QOI_OP_INDEX << 6) | index
// range from 0..63
pub(crate) const QOI_OP_INDEX: u8 = 0;

// (QOI_OP_DIFF << 6) | (dr << 4) | (dg << 2) | (db)
// range from 64..127
pub(crate) const QOI_OP_DIFF: u8 = 1;

// (QOI_OP_LUMA << 6) | dg
// range from 128..191
pub(crate) const QOI_OP_LUMA: u8 = 2;

// (QOI_OP_RUN << 6) | run
// range from 192..253
pub(crate) const QOI_OP_RUN: u8 = 3;

// bytes stream end
pub(crate) const QOI_END: u16 = 1 << 8;
