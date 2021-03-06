#[macro_use]
extern crate failure;

use std::cmp;

pub type Rgba = (u8, u8, u8, u8);
pub type Rgb = (u8, u8, u8);

static DEFAULT_DIFF_COLOR: Rgba = (255, 119, 119, 255);
static DEFAULT_ANTI_ALIASED_COLOR: Rgba = (243, 156, 18, 255);

#[derive(Debug, Fail)]
pub enum PixelmatchError {
    #[fail(display = "input buf length error. please input same length images.")]
    ImageLengthError,

    #[fail(display = "input buf format error. please input RGBA 24bit image data")]
    InvalidFormatError,
}

#[derive(Debug)]
pub struct PixelmatchResult {
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
) -> Result<PixelmatchResult, PixelmatchError> {
    if img1.len() != img2.len() {
        return Err(PixelmatchError::ImageLengthError);
    }
    if img1.len() % 4 != 0 {
        return Err(PixelmatchError::InvalidFormatError);
    }

    let options = if options.is_some() {
        options.expect("")
    } else {
        PixelmatchOption::default()
    };

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
                    && (anti_aliased(img1, x as usize, y as usize, dimensions, Some(img2))
                        || anti_aliased(img2, x as usize, y as usize, dimensions, Some(img1)))
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
    Ok(PixelmatchResult {
        diff_count,
        diff_image,
    })
}

fn draw_pixel(diff_buf: &mut Vec<u8>, pos: usize, rgba: Rgba) {
    diff_buf[pos + 0] = rgba.0;
    diff_buf[pos + 1] = rgba.1;
    diff_buf[pos + 2] = rgba.2;
    diff_buf[pos + 3] = rgba.3;
}

fn gray_pixel(img: &[u8], pos: usize) -> u8 {
    let a = img[pos + 3] as f32 / 255.0;
    let r = blend(img[pos + 0], a);
    let g = blend(img[pos + 1], a);
    let b = blend(img[pos + 2], a);
    rgb2y(r, g, b) as u8
}

// calculate color difference according to the paper "Measuring perceived color difference
// using YIQ NTSC transmission color space in mobile applications" by Y. Kotsarenko and F. Ramos
fn color_delta(img1: &[u8], img2: &[u8], pos1: usize, pos2: usize, only_brightness: bool) -> f32 {
    let a1 = img1[pos1 + 3] as f32 / 255.0;
    let a2 = img2[pos2 + 3] as f32 / 255.0;

    let r1 = blend(img1[pos1 + 0], a1);
    let g1 = blend(img1[pos1 + 1], a1);
    let b1 = blend(img1[pos1 + 2], a1);

    let r2 = blend(img2[pos2 + 0], a2);
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
    r as f32 * 0.29889531 + g as f32 * 0.58662247 + b as f32 * 0.11448223
}
fn rgb2i(r: u8, g: u8, b: u8) -> f32 {
    r as f32 * 0.59597799 - g as f32 * 0.27417610 - b as f32 * 0.32180189
}
fn rgb2q(r: u8, g: u8, b: u8) -> f32 {
    r as f32 * 0.21147017 - g as f32 * 0.52261711 + b as f32 * 0.31114694
}

// check if a pixel is likely a part of anti-aliasing;
// based on "Anti-aliased Pixel and Intensity Slope Detector" paper by V. Vysniauskas, 2009
// http://eejournal.ktu.lt/index.php/elt/article/view/10058/5000

fn anti_aliased(
    img1: &[u8],
    x1: usize,
    y1: usize,
    dimensions: (u32, u32),
    img2: Option<&[u8]>,
) -> bool {
    let x0 = cmp::max(x1 as i32 - 1, 0);
    let y0 = cmp::max(y1 as i32 - 1, 0);
    let x2 = cmp::min(x1 as i32 + 1, dimensions.0 as i32 - 1);
    let y2 = cmp::min(y1 as i32 + 1, dimensions.1 as i32 - 1);
    let pos = (y1 * dimensions.0 as usize + x1) * 4;
    let mut zeroes = 0;
    let mut positives = 0;
    let mut negatives = 0;
    let mut min = 0;
    let mut max = 0;
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;

    // go through 8 adjacent pixels
    for x in x0..x2 + 1 {
        for y in y0..y2 + 1 {
            if x == x1 as i32 && y == y1 as i32 {
                continue;
            }

            // brightness delta between the center pixel and adjacent one
            let delta = color_delta(
                img1,
                img1,
                pos,
                ((y * dimensions.0 as i32 + x) * 4) as usize,
                true,
            ) as i32;

            // count the number of equal, darker and brighter adjacent pixels
            if delta == 0 {
                zeroes += 1;
            } else if delta < 0 {
                negatives += 1;
            } else if delta > 0 {
                positives += 1;
            }

            // if found more than 2 equal siblings, it's definitely not anti-aliasing
            if zeroes > 2 {
                return false;
            }

            if img2.is_none() {
                continue;
            }

            // remember the darkest pixel
            if delta < min {
                min = delta;
                min_x = x;
                min_y = y;
            }
            // remember the brightest pixel
            if delta > max {
                max = delta;
                max_x = x;
                max_y = y;
            }
        }
    }

    if img2.is_none() {
        return true;
    }

    // if there are no both darker and brighter pixels among siblings, it's not anti-aliasing
    if negatives == 0 || positives == 0 {
        return false;
    }

    // if either the darkest or the brightest pixel has more than 2 equal siblings in both images
    // (definitely not anti-aliased), this pixel is anti-aliased
    (!anti_aliased(img1, min_x as usize, min_y as usize, dimensions, None)
        && !anti_aliased(
            img2.unwrap(),
            min_x as usize,
            min_y as usize,
            dimensions,
            None,
        ))
        || (!anti_aliased(img1, max_x as usize, max_y as usize, dimensions, None)
            && !anti_aliased(
                img2.unwrap(),
                max_x as usize,
                max_y as usize,
                dimensions,
                None,
            ))
}

#[test]
fn should_detect_1pixel_diff() {
    let img1 = vec![255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let img2 = vec![0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let result = pixelmatch(&img1, &img2, (2, 2), None).unwrap();
    assert_eq!(result.diff_count, 1);
    assert_eq!(
        result.diff_image,
        vec![255, 119, 119, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]
    );
}
