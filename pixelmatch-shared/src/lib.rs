pub type Rgba = (u8, u8, u8, u8);
pub type Rgb = (u8, u8, u8);

pub static DEFAULT_DIFF_COLOR: Rgba = (255, 119, 119, 255);
pub static DEFAULT_ANTI_ALIASED_COLOR: Rgba = (243, 156, 18, 255);

#[derive(Debug)]
pub enum PixelmatchError {
    ImageLengthError,
    InvalidFormatError,
}

impl std::fmt::Display for PixelmatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PixelmatchError::ImageLengthError => {
                f.write_str("input buf length error. please input same length images")
            }
            PixelmatchError::InvalidFormatError => {
                f.write_str("input buf format error. please input RGBA 24bit image data")
            }
        }
    }
}

impl std::error::Error for PixelmatchError {
    fn description(&self) -> &str {
        match *self {
            PixelmatchError::ImageLengthError => {
                "input buf length error. please input same length images"
            }
            PixelmatchError::InvalidFormatError => {
                "input buf format error. please input RGBA 24bit image data"
            }
        }
    }
}

#[derive(Debug)]
pub struct PixelmatchOutput {
    pub diff_count: usize,
    pub diff_image: Vec<u8>,
}

pub fn draw_pixel(diff_buf: &mut [u8], pos: usize, rgba: Rgba) {
    diff_buf[pos] = rgba.0;
    diff_buf[pos + 1] = rgba.1;
    diff_buf[pos + 2] = rgba.2;
    diff_buf[pos + 3] = rgba.3;
}

