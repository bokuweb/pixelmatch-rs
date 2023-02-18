#![no_std]
use core::cmp::{max, min};
use wasm_bindgen::prelude::*;

struct PixelmatchOption {
    pub include_anti_alias: bool,
    pub threshold: f32,
    pub diff_color: Rgba,
    pub anti_aliased_color: Rgba,
}

pub struct Rgba(u8, u8, u8, u8);

const IMAGE_LENGTH_ERROR: isize = -1;
const INVALID_FORMAT_ERROR: isize = -2;

#[wasm_bindgen]
pub fn pixelmatch(
    img1: &[u8],
    img2: &[u8],
    out: &mut [u8],
    width: u32,
    height: u32,
    include_anti_alias: bool,
    threshold: f32,
    diff_color_r: u8,
    diff_color_g: u8,
    diff_color_b: u8,
    diff_color_a: u8,
    anti_aliased_color_r: u8,
    anti_aliased_color_g: u8,
    anti_aliased_color_b: u8,
    anti_aliased_color_a: u8,
) -> isize {
    if img1.len() != img2.len() {
        return IMAGE_LENGTH_ERROR;
    }
    if img1.len() % 4 != 0 {
        return INVALID_FORMAT_ERROR;
    }

    let options = PixelmatchOption {
        include_anti_alias,
        threshold,
        diff_color: Rgba(diff_color_r, diff_color_g, diff_color_b, diff_color_a),
        anti_aliased_color: Rgba(
            anti_aliased_color_r,
            anti_aliased_color_g,
            anti_aliased_color_b,
            anti_aliased_color_a,
        ),
    };

    // maximum acceptable square distance between two colors;
    // 35215 is the maximum possible value for the YIQ difference metric
    let threshold = options.threshold;
    let max_delta = 35215.0 * threshold * threshold;
    let mut diff_count = 0;

    for y in 0..height {
        for x in 0..width {
            let pos = ((y * width + x) * 4) as usize;
            // squared YUV distance between colors at this pixel position
            let delta = color_delta(img1, img2, pos, pos, false);
            if f32::abs(delta) > max_delta {
                // check it's a real rendering difference or just anti-aliasing
                if !options.include_anti_alias
                    && (anti_aliased(img1, x as usize, y as usize, (width, height), img2)
                        || anti_aliased(img2, x as usize, y as usize, (width, height), img1))
                {
                    // one of the pixels is anti-aliasing; draw as yellow and do not count as difference
                    draw_pixel(out, pos, &options.anti_aliased_color);
                } else {
                    // found substantial difference not caused by anti-aliasing; draw it as red
                    draw_pixel(out, pos, &options.diff_color);
                    diff_count += 1;
                }
            } else {
                // pixels are similar; draw background as grayscale image blended with white
                let y = blend(gray_pixel(img1, pos), 0.1);
                draw_pixel(out, pos, &Rgba(y as u8, y as u8, y as u8, 255));
            }
        }
    }
    diff_count
}

fn draw_pixel(diff_buf: &mut [u8], pos: usize, rgba: &Rgba) {
    unsafe {
        *diff_buf.get_unchecked_mut(pos) = rgba.0;
        *diff_buf.get_unchecked_mut(pos + 1) = rgba.1;
        *diff_buf.get_unchecked_mut(pos + 2) = rgba.2;
        *diff_buf.get_unchecked_mut(pos + 3) = rgba.3;
    }
}

fn gray_pixel(img: &[u8], pos: usize) -> u8 {
    unsafe {
        let a = *img.get_unchecked(pos + 3) as f32 / 255.0;
        let r = blend(*img.get_unchecked(pos), a);
        let g = blend(*img.get_unchecked(pos + 1), a);
        let b = blend(*img.get_unchecked(pos + 2), a);
        rgb2y(r, g, b) as u8
    }
}

// calculate color difference according to the paper "Measuring perceived color difference
// using YIQ NTSC transmission color space in mobile applications" by Y. Kotsarenko and F. Ramos
fn color_delta(img1: &[u8], img2: &[u8], pos1: usize, pos2: usize, only_brightness: bool) -> f32 {
    unsafe {
        let r1 = *img1.get_unchecked(pos1);
        let g1 = *img1.get_unchecked(pos1 + 1);
        let b1 = *img1.get_unchecked(pos1 + 2);
        let a1 = *img1.get_unchecked(pos1 + 3);

        let r2 = *img2.get_unchecked(pos2);
        let g2 = *img2.get_unchecked(pos2 + 1);
        let b2 = *img2.get_unchecked(pos2 + 2);
        let a2 = *img2.get_unchecked(pos2 + 3);

        if a1 == a2 && r1 == r2 && g1 == g2 && b1 == b2 {
            return 0.0;
        }

        let (r1, g1, b1) = if a1 < 255 {
            let a1 = a1 as f32 / 255.0;
            (blend(r1, a1), blend(g1, a1), blend(b1, a1))
        } else {
            (r1 as f32, g1 as f32, b1 as f32)
        };

        let (r2, g2, b2) = if a2 < 255 {
            let a2 = a2 as f32 / 255.0;
            (blend(r2, a2), blend(g2, a2), blend(b2, a2))
        } else {
            (r2 as f32, g2 as f32, b2 as f32)
        };

        let y1 = rgb2y(r1, g1, b1);
        let y2 = rgb2y(r2, g2, b2);
        let y = y1 - y2;

        if only_brightness {
            return y;
        }

        let i = rgb2i(r1, g1, b1) - rgb2i(r2, g2, b2);
        let q = rgb2q(r1, g1, b1) - rgb2q(r2, g2, b2);

        let delta = 0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q;
        if y1 > y2 {
            -delta
        } else {
            delta
        }
    }
}

// blend semi-transparent color with white
fn blend(c: u8, a: f32) -> f32 {
    255.0 + ((c as f32 - 255.0) as f32) * a
}

fn rgb2y(r: f32, g: f32, b: f32) -> f32 {
    r * 0.29889531 + g * 0.58662247 + b * 0.11448223
}

fn rgb2i(r: f32, g: f32, b: f32) -> f32 {
    r * 0.59597799 - g * 0.2741761 - b * 0.32180189
}

fn rgb2q(r: f32, g: f32, b: f32) -> f32 {
    r * 0.21147017 - g * 0.52261711 + b * 0.31114694
}

/// check if a pixel is likely a part of anti-aliasing;
/// based on "Anti-aliased Pixel and Intensity Slope Detector" paper by V. Vysniauskas, 2009
/// http://eejournal.ktu.lt/index.php/elt/article/view/10058/5000
fn anti_aliased(img1: &[u8], x1: usize, y1: usize, dimensions: (u32, u32), img2: &[u8]) -> bool {
    let x0 = max(x1 as i32 - 1, 0) as usize;
    let y0 = max(y1 as i32 - 1, 0) as usize;

    let x2 = min(x1 as i32 + 1, dimensions.0 as i32 - 1) as usize;
    let y2 = min(y1 as i32 + 1, dimensions.1 as i32 - 1) as usize;

    let pos = (y1 * dimensions.0 as usize + x1) * 4;
    let mut zeroes = if x1 == x0 || x1 == x2 || y1 == y0 || y1 == y2 {
        1
    } else {
        0
    };

    let mut min = 0.0;
    let mut max = 0.0;
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
            );

            // count the number of equal, darker and brighter adjacent pixels
            if delta == 0.0 {
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
    if min == 0.0 || max == 0.0 {
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
    let x0 = max(x1 - 1, 0);
    let y0 = max(y1 - 1, 0);
    let x2 = min(x1 + 1, width as usize - 1);
    let y2 = min(y1 + 1, height as usize - 1);
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

            let eq = unsafe {
                *img.get_unchecked(pos) == *img.get_unchecked(pos2)
                    && *img.get_unchecked(pos + 1) == *img.get_unchecked(pos2 + 1)
                    && *img.get_unchecked(pos + 2) == *img.get_unchecked(pos2 + 2)
                    && *img.get_unchecked(pos + 3) == *img.get_unchecked(pos2 + 3)
            };
            if eq {
                zeroes += 1;
            }

            if zeroes > 2 {
                return true;
            }
        }
    }
    false
}
