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

impl Default for PixelmatchOption {
    fn default() -> Self {
        Self {
            include_anti_alias: false,
            threshold: 0.1,
            diff_color: DEFAULT_DIFF_COLOR,
            anti_aliased_color: DEFAULT_ANTI_ALIASED_COLOR,
        }
    }
}

pub fn draw_pixel(diff_buf: &mut [u8], pos: usize, rgba: Rgba) {
    diff_buf[pos] = rgba.0;
    diff_buf[pos + 1] = rgba.1;
    diff_buf[pos + 2] = rgba.2;
    diff_buf[pos + 3] = rgba.3;
}

#[derive(Debug)]
pub struct PixelmatchOption {
    pub include_anti_alias: bool,
    pub threshold: f32,
    pub diff_color: Rgba,
    pub anti_aliased_color: Rgba,
}
