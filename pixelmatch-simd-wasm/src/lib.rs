#![no_std]
use core::arch::wasm32::*;
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

const V_Y: v128 = f32x4(0.29889531, 0.58662247, 0.11448223, 1.0);
const V_I: v128 = f32x4(0.59597799, -0.27417610, -0.32180189, 1.0);
const V_Q: v128 = f32x4(0.21147017, -0.52261711, 0.31114694, 1.0);
const V_WHITE: v128 = f32x4(255.0, 255.0, 255.0, 0.0);
const V_DELTA: v128 = f32x4(0.5053, 0.299, 0.1957, 0.0);

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

            let rgba1 = unsafe {
                f32x4(
                    *img1.get_unchecked(pos) as f32,
                    *img1.get_unchecked(pos + 1) as f32,
                    *img1.get_unchecked(pos + 2) as f32,
                    *img1.get_unchecked(pos + 3) as f32,
                )
            };

            let rgba2 = unsafe {
                f32x4(
                    *img2.get_unchecked(pos) as f32,
                    *img2.get_unchecked(pos + 1) as f32,
                    *img2.get_unchecked(pos + 2) as f32,
                    *img2.get_unchecked(pos + 3) as f32,
                )
            };

            // squared YUV distance between colors at this pixel position
            let delta = color_delta(rgba1, rgba2, false);
            if f32::abs(delta) > max_delta {
                // check it's a real rendering difference or just anti-aliasing
                if !options.include_anti_alias
                    && (anti_aliased(img1, x as usize, y as usize, rgba1, (width, height), img2)
                        || anti_aliased(img2, x as usize, y as usize, rgba2, (width, height), img1))
                {
                    // one of the pixels is anti-aliasing; draw as yellow and do not count as difference
                    draw_pixel(out, pos, &options.anti_aliased_color);
                } else {
                    // found substantial difference not caused by anti-aliasing; draw it as red
                    draw_pixel(out, pos, &options.diff_color);
                    diff_count += 1;
                }
            } else {
                let c = gray_pixel(rgba1);
                // pixels are similar; draw background as grayscale image blended with white
                let y = blend_u8(c as u8, 0.1);
                draw_pixel(out, pos, &Rgba(y, y, y, 255));
            }
        }
    }
    diff_count
}

fn gray_pixel(rgba: v128) -> f32 {
    let a = f32x4_extract_lane::<3>(rgba);
    let rgba = blend(rgba, a);
    rgb2y(rgba)
}

fn sum_rgb(v: v128) -> f32 {
    f32x4_extract_lane::<0>(v) + f32x4_extract_lane::<1>(v) + f32x4_extract_lane::<2>(v)
}

// calculate color difference according to the paper "Measuring perceived color difference
// using YIQ NTSC transmission color space in mobile applications" by Y. Kotsarenko and F. Ramos
fn color_delta(rgba1: v128, rgba2: v128, only_brightness: bool) -> f32 {
    if u32x4_all_true(f32x4_convert_u32x4(f32x4_eq(rgba1, rgba2))) {
        return 0.0;
    }
    let a1 = f32x4_extract_lane::<3>(rgba1);
    let a2 = f32x4_extract_lane::<3>(rgba2);

    let rgba1 = if a1 < 255.0 { blend(rgba1, a1) } else { rgba1 };
    let rgba2 = if a2 < 255.0 { blend(rgba2, a2) } else { rgba2 };

    let y1 = rgb2y(rgba1);
    let y2 = rgb2y(rgba2);

    let y = y1 - y2;

    if only_brightness {
        return y;
    }

    let i = rgb2i(rgba1) - rgb2i(rgba2);
    let q = rgb2q(rgba1) - rgb2q(rgba2);

    let v = f32x4(y, i, q, 0.0);
    let delta = sum_rgb(f32x4_mul(f32x4_mul(v, v), V_DELTA));
    // let delta = 0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q;

    if y1 > y2 {
        -delta
    } else {
        delta
    }
}

fn blend_u8(c: u8, a: f32) -> u8 {
    (255.0 + ((c as f32 - 255.0) as f32) * a) as u8
}

/// blend semi-transparent color with white
fn blend(rgba: v128, a: f32) -> v128 {
    let a = a / 255.0;
    f32x4_add(
        f32x4_mul(f32x4_sub(rgba, V_WHITE), f32x4(a, a, a, 1.0)),
        V_WHITE,
    )
}

fn rgb2y(px: v128) -> f32 {
    sum_rgb(f32x4_mul(px, V_Y))
}

fn rgb2i(rgba: v128) -> f32 {
    sum_rgb(f32x4_mul(rgba, V_I))
}

fn rgb2q(rgba: v128) -> f32 {
    sum_rgb(f32x4_mul(rgba, V_Q))
}

// check if a pixel is likely a part of anti-aliasing;
// based on "Anti-aliased Pixel and Intensity Slope Detector" paper by V. Vysniauskas, 2009
// http://eejournal.ktu.lt/index.php/elt/article/view/10058/5000
fn anti_aliased(
    img1: &[u8],
    x1: usize,
    y1: usize,
    rgba1: v128,
    dimensions: (u32, u32),
    img2: &[u8],
) -> bool {
    let x0 = max(x1 as i32 - 1, 0) as usize;
    let y0 = max(y1 as i32 - 1, 0) as usize;

    let x2 = min(x1 as i32 + 1, dimensions.0 as i32 - 1) as usize;
    let y2 = min(y1 as i32 + 1, dimensions.1 as i32 - 1) as usize;

    // let pos = (y1 * dimensions.0 as usize + x1) * 4;
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

    // let rgba1 = unsafe {
    //     f32x4(
    //         *img1.get_unchecked(pos) as f32,
    //         *img1.get_unchecked(pos + 1) as f32,
    //         *img1.get_unchecked(pos + 2) as f32,
    //         *img1.get_unchecked(pos + 3) as f32,
    //     )
    // };

    // go through 8 adjacent pixels
    for x in x0..=x2 {
        for y in y0..=y2 {
            if x == x1 && y == y1 {
                continue;
            }

            let pos2 = ((y * width as usize + x) * 4) as usize;
            let rgba2 = unsafe {
                f32x4(
                    *img1.get_unchecked(pos2) as f32,
                    *img1.get_unchecked(pos2 + 1) as f32,
                    *img1.get_unchecked(pos2 + 2) as f32,
                    *img1.get_unchecked(pos2 + 3) as f32,
                )
            };

            // brightness delta between the center pixel and adjacent one
            let delta = color_delta(rgba1, rgba2, true);

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

    // if either the darkest or the brightest pixel has more than 2 equal siblings in both images
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

    // let rgba1 = unsafe {
    //     f32x4(
    //         *img.get_unchecked(pos) as f32,
    //         *img.get_unchecked(pos + 1) as f32,
    //         *img.get_unchecked(pos + 2) as f32,
    //         *img.get_unchecked(pos + 3) as f32,
    //     )
    // };

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

            // let pos2 = (y * width as usize + x) * 4;
            // let rgba2 = unsafe {
            //     f32x4(
            //         *img.get_unchecked(pos2) as f32,
            //         *img.get_unchecked(pos2 + 1) as f32,
            //         *img.get_unchecked(pos2 + 2) as f32,
            //         *img.get_unchecked(pos2 + 3) as f32,
            //     )
            // };
            //
            // if u32x4_all_true(f32x4_convert_u32x4(f32x4_eq(rgba1, rgba2))) {
            //     zeroes += 1;
            // }

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

fn draw_pixel(diff_buf: &mut [u8], pos: usize, rgba: &Rgba) {
    unsafe {
        *diff_buf.get_unchecked_mut(pos) = rgba.0;
        *diff_buf.get_unchecked_mut(pos + 1) = rgba.1;
        *diff_buf.get_unchecked_mut(pos + 2) = rgba.2;
        *diff_buf.get_unchecked_mut(pos + 3) = rgba.3;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn should_detect_1pixel_diff() {
        let img1: [u8; 16] = [255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let img2: [u8; 16] = [0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut out: [u8; 16] = [0; 16];

        let result = pixelmatch(
            &img1, &img2, &mut out, 2, 2, true, 0.1, 255, 119, 119, 255, 243, 156, 18, 255,
        );
        assert_eq!(result, 1);
        assert_eq!(
            out,
            [255, 119, 119, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]
        );
        assert!(true);
    }
}
