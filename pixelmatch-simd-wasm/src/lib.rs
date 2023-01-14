#![no_std]

use core::arch::wasm32::*;
use core::cmp;
use core::result::Result;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: usize);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    let img1: [u8; 16] = [255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let img2: [u8; 16] = [0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut out: [u8; 16] = [0; 16];

    let result = pixelmatch(&img1, &img2, &mut out, 2, 2, None).unwrap();
    alert(result);
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Rgba(u8, u8, u8, u8);

#[wasm_bindgen]
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
            diff_color: Rgba(255, 119, 119, 255),
            anti_aliased_color: Rgba(243, 156, 18, 255),
        }
    }
}

pub enum PixelmatchError {
    ImageLengthError = 0x00,
    InvalidFormatError = 0x01,
}

#[wasm_bindgen(catch)]
pub fn pixelmatch(
    img1: &[u8],
    img2: &[u8],
    out: &mut [u8],
    width: u32,
    height: u32,
    options: Option<PixelmatchOption>,
) -> Result<usize, usize> {
    if img1.len() != img2.len() {
        return Err(PixelmatchError::ImageLengthError as usize);
    }
    if img1.len() % 4 != 0 {
        return Err(PixelmatchError::InvalidFormatError as usize);
    }

    let options = options.unwrap_or_default();

    // maximum acceptable square distance between two colors;
    // 35215 is the maximum possible value for the YIQ difference metric
    let threshold = options.threshold;
    let max_delta = 35215.0 * threshold * threshold;
    let mut diff_count = 0;

    for y in 0..height {
        for x in 0..width {
            let pos = ((y * width + x) * 4) as usize;

            let rgba1 = f32x4(
                img1[pos] as f32,
                img1[pos + 1] as f32,
                img1[pos + 2] as f32,
                img1[pos + 3] as f32,
            );

            let rgba2 = f32x4(
                img2[pos] as f32,
                img2[pos + 1] as f32,
                img2[pos + 2] as f32,
                img2[pos + 3] as f32,
            );

            // squared YUV distance between colors at this pixel position
            let delta = color_delta(rgba1, rgba2, false);
            if delta > max_delta {
                // check it's a real rendering difference or just anti-aliasing
                if options.include_anti_alias
                    && (anti_aliased(img1, x as usize, y as usize, (width, height), img2)
                        || anti_aliased(img2, x as usize, y as usize, (width, height), img1))
                {
                    // one of the pixels is anti-aliasing; draw as yellow and do not count as difference
                    draw_pixel(out, pos, options.anti_aliased_color);
                } else {
                    // found substantial difference not caused by anti-aliasing; draw it as red
                    draw_pixel(out, pos, options.diff_color);
                    diff_count += 1;
                }
            } else {
                let c = gray_pixel(rgba1);
                // pixels are similar; draw background as grayscale image blended with white
                let y = (255.0 + ((c as i32 - 255) as f32) * 0.1) as u8;
                draw_pixel(out, pos, Rgba(y, y, y, 255));
            }
        }
    }
    Ok(diff_count)
}

fn gray_pixel(rgba: v128) -> u8 {
    let rgba = blend(rgba);
    rgb2y(rgba) as u8
}

fn sum_rgb(v: v128) -> f32 {
    f32x4_extract_lane::<0>(v) + f32x4_extract_lane::<1>(v) + f32x4_extract_lane::<2>(v)
}

// calculate color difference according to the paper "Measuring perceived color difference
// using YIQ NTSC transmission color space in mobile applications" by Y. Kotsarenko and F. Ramos
fn color_delta(rgba1: v128, rgba2: v128, only_brightness: bool) -> f32 {
    let y1 = rgb2y(rgba1);
    let y2 = rgb2y(rgba2);

    let y = y1 - y2;

    if only_brightness {
        return y;
    }

    let i = rgb2i(rgba1) - rgb2i(rgba2);

    let q = rgb2q(rgba1) - rgb2q(rgba2);

    0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q
}

/// blend semi-transparent color with white
fn blend(rgba: v128) -> v128 {
    let a = (f32x4_extract_lane::<3>(rgba) / 255.0);
    f32x4_add(
        f32x4_mul(
            f32x4_sub(rgba, f32x4(255.0, 255.0, 255.0, 0.0)),
            f32x4(a, a, a, 1.0),
        ),
        f32x4(255.0, 255.0, 255.0, 0.0),
    )
}

fn rgb2i(rgba: v128) -> f32 {
    sum_rgb(f32x4_mul(
        rgba,
        f32x4(0.59597799, -0.27417610, -0.32180189, 1.0),
    ))
}

fn rgb2q(rgba: v128) -> f32 {
    sum_rgb(f32x4_mul(
        rgba,
        f32x4(0.21147017, -0.52261711, 0.31114694, 1.0),
    ))
}

fn rgb2y(px: v128) -> f32 {
    sum_rgb(f32x4_mul(
        px,
        f32x4(0.29889531, 0.58662247, 0.11448223, 1.0),
    ))
}

// check if a pixel is likely a part of anti-aliasing;
// based on "Anti-aliased Pixel and Intensity Slope Detector" paper by V. Vysniauskas, 2009
// http://eejournal.ktu.lt/index.php/elt/article/view/10058/5000
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

            let rgba1 = f32x4(
                img1[pos] as f32,
                img1[pos + 1] as f32,
                img1[pos + 2] as f32,
                img1[pos + 3] as f32,
            );

            let pos2 = ((y * width as usize + x) * 4) as usize;
            let rgba2 = f32x4(
                img1[pos2] as f32,
                img1[pos2 + 1] as f32,
                img1[pos2 + 2] as f32,
                img1[pos2 + 3] as f32,
            );

            // brightness delta between the center pixel and adjacent one
            let delta = color_delta(rgba1, rgba2, true) as i32;

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

    // if either the darkest or the brightest pixel has more than 2 equal siblings in both images
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

fn draw_pixel(diff_buf: &mut [u8], pos: usize, rgba: Rgba) {
    diff_buf[pos] = rgba.0;
    diff_buf[pos + 1] = rgba.1;
    diff_buf[pos + 2] = rgba.2;
    diff_buf[pos + 3] = rgba.3;
}
