use core::cmp;

pub type Rgba = (u8, u8, u8, u8);
pub type Rgb = (u8, u8, u8);

pub static DEFAULT_DIFF_COLOR: Rgba = (255, 119, 119, 255);
pub static DEFAULT_ANTI_ALIASED_COLOR: Rgba = (243, 156, 18, 255);

#[derive(Debug)]
pub enum PixelmatchError {
    ImageLengthError,
    InvalidFormatError,
}

const IMAGE_LENGTH_ERROR_MESSAGE: &str = "input buf length error. please input same length images";
const INVALID_FORMAT_ERROR_MESSAGE: &str =
    "input buf format error. please input RGBA 24bit image data";

impl std::fmt::Display for PixelmatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PixelmatchError::ImageLengthError => f.write_str(IMAGE_LENGTH_ERROR_MESSAGE),
            PixelmatchError::InvalidFormatError => f.write_str(INVALID_FORMAT_ERROR_MESSAGE),
        }
    }
}

impl std::error::Error for PixelmatchError {
    fn description(&self) -> &str {
        match *self {
            PixelmatchError::ImageLengthError => IMAGE_LENGTH_ERROR_MESSAGE,
            PixelmatchError::InvalidFormatError => INVALID_FORMAT_ERROR_MESSAGE,
        }
    }
}

#[derive(Debug)]
pub struct PixelmatchOutput {
    pub diff_count: usize,
    pub diff_image: Vec<u8>,
}

#[derive(Debug)]
pub struct PixelmatchOption {
    pub include_anti_alias: bool,
    pub threshold: f32,
    pub diff_color: Rgba,
    pub anti_aliased_color: Rgba,
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

pub fn pixelmatch(
    img1: &[u8],
    img2: &[u8],
    dimensions: (u32, u32),
    options: Option<PixelmatchOption>,
) -> Result<PixelmatchOutput, PixelmatchError> {
    if img1.len() != img2.len() {
        return Err(PixelmatchError::ImageLengthError);
    }
    if img1.len() % 4 != 0 {
        return Err(PixelmatchError::InvalidFormatError);
    }

    let options = options.unwrap_or_default();

    // maximum acceptable square distance between two colors;
    // 35215 is the maximum possible value for the YIQ difference metric
    let threshold = options.threshold;
    let max_delta = 35215.0 * threshold * threshold;
    let mut diff_count = 0;
    let mut diff_image: Vec<u8> = vec![0; img1.len()];

    for y in 0..dimensions.1 {
        for x in 0..dimensions.0 {
            let pos = ((y * dimensions.0 + x) * 4) as usize;
            // squared YUV distance between colors at this pixel position
            let delta = color_delta(img1, img2, pos, pos, false);
            if delta > max_delta {
                // check it's a real rendering difference or just anti-aliasing
                if options.include_anti_alias
                    && (anti_aliased(img1, x as usize, y as usize, dimensions, img2)
                        || anti_aliased(img2, x as usize, y as usize, dimensions, img1))
                {
                    // one of the pixels is anti-aliasing; draw as yellow and do not count as difference
                    draw_pixel(&mut diff_image, pos, options.anti_aliased_color);
                } else {
                    // found substantial difference not caused by anti-aliasing; draw it as red
                    draw_pixel(&mut diff_image, pos, options.diff_color);
                    diff_count += 1;
                }
            } else {
                // pixels are similar; draw background as grayscale image blended with white
                let y = blend(gray_pixel(img1, pos), 0.1);
                draw_pixel(&mut diff_image, pos, (y, y, y, 255));
            }
        }
    }
    Ok(PixelmatchOutput {
        diff_count,
        diff_image,
    })
}

fn draw_pixel(diff_buf: &mut [u8], pos: usize, rgba: Rgba) {
    diff_buf[pos] = rgba.0;
    diff_buf[pos + 1] = rgba.1;
    diff_buf[pos + 2] = rgba.2;
    diff_buf[pos + 3] = rgba.3;
}

fn gray_pixel(img: &[u8], pos: usize) -> u8 {
    let a = img[pos + 3] as f32 / 255.0;
    let r = blend(img[pos], a);
    let g = blend(img[pos + 1], a);
    let b = blend(img[pos + 2], a);
    rgb2y(r, g, b) as u8
}

// calculate color difference according to the paper "Measuring perceived color difference
// using YIQ NTSC transmission color space in mobile applications" by Y. Kotsarenko and F. Ramos
fn color_delta(img1: &[u8], img2: &[u8], pos1: usize, pos2: usize, only_brightness: bool) -> f32 {
    let a1 = img1[pos1 + 3] as f32 / 255.0;
    let a2 = img2[pos2 + 3] as f32 / 255.0;

    let r1 = blend(img1[pos1], a1);
    let g1 = blend(img1[pos1 + 1], a1);
    let b1 = blend(img1[pos1 + 2], a1);

    let r2 = blend(img2[pos2], a2);
    let g2 = blend(img2[pos2 + 1], a2);
    let b2 = blend(img2[pos2 + 2], a2);

    let y = rgb2y(r1, g1, b1) - rgb2y(r2, g2, b2);

    if only_brightness {
        return y;
    }

    let i = rgb2i(r1, g1, b1) - rgb2i(r2, g2, b2);
    let q = rgb2q(r1, g1, b1) - rgb2q(r2, g2, b2);

    0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q
}

// blend semi-transparent color with white
fn blend(c: u8, a: f32) -> u8 {
    (255.0 + ((c as i32 - 255) as f32) * a) as u8
}

fn rgb2y(r: u8, g: u8, b: u8) -> f32 {
    r as f32 * 0.298_895_3 + g as f32 * 0.586_622_4 + b as f32 * 0.114_482_23
}
fn rgb2i(r: u8, g: u8, b: u8) -> f32 {
    r as f32 * 0.595_977_99 - g as f32 * 0.274_176_1 - b as f32 * 0.321_801_8
}
fn rgb2q(r: u8, g: u8, b: u8) -> f32 {
    r as f32 * 0.211_470_17 - g as f32 * 0.522_617_1 + b as f32 * 0.311_146_9
}

/// check if a pixel is likely a part of anti-aliasing;
/// based on "Anti-aliased Pixel and Intensity Slope Detector" paper by V. Vysniauskas, 2009
/// http://eejournal.ktu.lt/index.php/elt/article/view/10058/5000
fn anti_aliased(img1: &[u8], x1: usize, y1: usize, dimensions: (u32, u32), img2: &[u8]) -> bool {
    let x0 = cmp::max(x1 as i32 - 1, 0) as usize;
    let y0 = cmp::max(y1 as i32 - 1, 0) as usize;

    let x2 = cmp::min(x1 as i32 + 1, dimensions.0 as i32 - 1) as usize;
    let y2 = cmp::min(y1 as i32 + 1, dimensions.1 as i32 - 1) as usize;

    let pos = (y1 * dimensions.0 as usize + x1) * 4;
    let mut zeroes = if x1 == x0 || x1 == x2 || y1 == y0 || y1 == y2 {
        1
    } else {
        0
    };

    let mut min = 0;
    let mut max = 0;
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;

    let (width, height) = dimensions;

    // go through 8 adjacent pixels
    for x in x0..=x2 {
        for y in y0..=y2 {
            if x == x1 && y == y1 {
                continue;
            }

            // brightness delta between the center pixel and adjacent one
            let delta = color_delta(
                img1,
                img1,
                pos,
                ((y * width as usize + x) * 4) as usize,
                true,
            ) as i32;

            // count the number of equal, darker and brighter adjacent pixels
            if delta == 0 {
                zeroes += 1;
                // if found more than 2 equal siblings, it's definitely not anti-aliasing
                if zeroes > 2 {
                    return false;
                }
                // remember the darkest pixel
            } else if delta < min {
                min = delta;
                min_x = x;
                min_y = y;
                // remember the brightest pixel
            } else if delta > max {
                max = delta;
                max_x = x;
                max_y = y;
            }
        }
    }

    // if there are no both darker and brighter pixels among siblings, it's not anti-aliasing
    if min == 0 || max == 0 {
        return false;
    }

    // if either the darkest or the brightest pixel has 3+ equal siblings in both images
    // (definitely not anti-aliased), this pixel is anti-aliased
    (has_many_siblings(img1, min_x, min_y, width, height)
        && has_many_siblings(img2, min_x, min_y, width, height))
        || (has_many_siblings(img1, max_x, max_y, width, height)
            && has_many_siblings(img2, max_x, max_y, width, height))
}

/// check if a pixel has 3+ adjacent pixels of the same color.
fn has_many_siblings(img: &[u8], x1: usize, y1: usize, width: u32, height: u32) -> bool {
    let x0 = cmp::max(x1 - 1, 0);
    let y0 = cmp::max(y1 - 1, 0);
    let x2 = cmp::min(x1 + 1, width as usize - 1);
    let y2 = cmp::min(y1 + 1, height as usize - 1);
    let pos = (y1 * width as usize + x1) * 4;

    let mut zeroes = if x1 == x0 || x1 == x2 || y1 == y0 || y1 == y2 {
        1
    } else {
        0
    };

    // go through 8 adjacent pixels
    for x in x0..=x2 {
        for y in y0..=y2 {
            if x == x1 && y == y1 {
                continue;
            }

            let pos2 = (y * width as usize + x) * 4;

            if img[pos] == img[pos2]
                && img[pos + 1] == img[pos2 + 1]
                && img[pos + 2] == img[pos2 + 2]
                && img[pos + 3] == img[pos2 + 3]
            {
                zeroes += 1;
            }

            if zeroes > 2 {
                return true;
            }
        }
    }
    false
}

#[test]
fn should_detect_1pixel_diff() {
    let img1 = vec![255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let img2 = vec![0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let result = pixelmatch(&img1, &img2, (2, 2), None).unwrap();
    assert_eq!(result.diff_count, 1);
    assert_eq!(
        result.diff_image,
        vec![255, 119, 119, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]
    );
}
