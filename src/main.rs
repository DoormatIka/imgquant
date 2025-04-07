
pub mod core;

use core::octree::{add_colors, div_colors, mul_colors, Octree};
use std::{cmp, env, fs::File, path::{self, Path, PathBuf}, time::{Duration, Instant}, u32, u8};
use getargs::{Arg, Options};
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgb, RgbImage, Rgba, RgbaImage};

fn bw_quant_basic_dithering(source: &DynamicImage, destination: &mut RgbaImage) {
    let mut color_error: i16 = 0;
    let image_width = source.width();
    for pixel in source.pixels() {
        let x = pixel.0;
        let y = pixel.1;
        let rgba: [u8; 4] = pixel.2.0;
        let grayscale = (f32::from(rgba[0]) * 0.2126 
            + f32::from(rgba[1]) * 0.7152 
            + f32::from(rgba[2]) * 0.0722)
        .round()
        .clamp(0.0, 255.0) as i16;
        let corrected_grayscale = (grayscale + color_error).clamp(0, 255);
        let bw_color: u8 = if corrected_grayscale <= 127 {
            0
        } else {
            255
        };
        color_error = corrected_grayscale - i16::from(bw_color);

        destination.put_pixel(x, y, Rgba([bw_color, bw_color, bw_color, rgba[3]]));

        if x > image_width {
            color_error = 0;
        }
    }
}

// cool as hell filter i stumbled into accidentally.
fn bw_quant_line_filter(source: &DynamicImage, destination: &mut RgbImage) {
    let mut color_error: i16 = 0;
    let image_width = source.width();
    
    for (x, y, pixel) in source.pixels() {
        let rgba: [u8; 4] = pixel.0;
        let grayscale = (f32::from(rgba[0]) * 0.2126 
            + f32::from(rgba[1]) * 0.7152 
            + f32::from(rgba[2]) * 0.0722)
        .round()
        .clamp(0.0, 255.0) as i16;

        let corrected_grayscale = grayscale - color_error;

        let bw_color: u8 = if grayscale <= 127 + color_error {
            0
        } else {
            255
        };
        color_error = (corrected_grayscale - i16::from(bw_color)).clamp(0, 255);

        destination.put_pixel(x, y, Rgb([bw_color, bw_color, bw_color]));

        if x > image_width {
            color_error = 0;
        }
    }
}


fn bw_quant_floyd_seinberg_dither(source: &DynamicImage, destination: &mut RgbaImage) {
    let image_width = source.width() as usize;

    let mut current_errors = vec![0i16; image_width + 1];
    let mut forward_errors = vec![0i16; image_width + 1];
    // let mut current_errors: Vec<i16> = Vec::with_capacity(image_width as usize);
    // let mut forward_errors: Vec<i16> = Vec::with_capacity(image_width as usize);

    for pixel in source.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let rgba: [u8; 4] = pixel.2.0;
        let grayscale = (f32::from(rgba[0]) * 0.2126 
            + f32::from(rgba[1]) * 0.7152 
            + f32::from(rgba[2]) * 0.0722)
        .round()
        .clamp(0.0, 255.0) as i16;
        let corrected_grayscale = (grayscale + current_errors[x as usize]).clamp(0, 255);
        let bw_color: u8 = if corrected_grayscale <= 127 { 0 } else { 255 };

        let color_error = corrected_grayscale - i16::from(bw_color);

        // https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html
        // https://www.youtube.com/watch?v=ico4fJfohMQ
        let forward_x = (x + 1).clamp(0, image_width) as usize;
        let behind_x = if x <= 0 { 0 } else { (x - 1).clamp(0, image_width) as usize };
        current_errors[forward_x] += color_error * 7 / 16;
        forward_errors[behind_x] += color_error * 3 / 16;
        forward_errors[x] += color_error * 5 / 16;
        forward_errors[forward_x] += color_error / 16;
        
        destination.put_pixel(x as u32, y as u32, Rgba([bw_color, bw_color, bw_color, rgba[3]]));

        if x >= image_width - 1 {
            current_errors.clone_from_slice(&forward_errors);
            forward_errors.fill(0);
        }
    }
}

fn bw_quant_sierra_lite_dither(source: &DynamicImage, destination: &mut RgbaImage) {
    let image_width = source.width() as usize;

    let mut current_errors = vec![0i16; image_width + 1];
    let mut forward_errors = vec![0i16; image_width + 1];

    for pixel in source.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let rgba: [u8; 4] = pixel.2.0;
        let grayscale = (f32::from(rgba[0]) * 0.2126 
            + f32::from(rgba[1]) * 0.7152 
            + f32::from(rgba[2]) * 0.0722)
        .round()
        .clamp(0.0, 255.0) as i16;
        let corrected_grayscale = (grayscale + current_errors[x as usize]).clamp(0, 255);
        let bw_color: u8 = if corrected_grayscale <= 127 { 0 } else { 255 };

        let color_error = corrected_grayscale - i16::from(bw_color);

        // https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html
        // https://www.youtube.com/watch?v=ico4fJfohMQ
        let forward_x = (x + 1).clamp(0, image_width) as usize;
        let behind_x = if x <= 0 { 0 } else { (x - 1).clamp(0, image_width) as usize };
        current_errors[forward_x] += color_error * 2 / 4;
        forward_errors[behind_x] += color_error / 4;
        forward_errors[x] += color_error / 4;
        
        destination.put_pixel(x as u32, y as u32, Rgba([bw_color, bw_color, bw_color, rgba[3]]));

        if x >= image_width - 1 {
            current_errors.clone_from_slice(&forward_errors);
            forward_errors.fill(0);
        }
    }
}

// black and white version!
fn sierra_lite(source: &DynamicImage, destination: &mut RgbaImage) {
    let image_width = source.width() as usize;

    let mut current_errors = vec![0i16; image_width + 1];
    let mut forward_errors = vec![0i16; image_width + 1];

    for pixel in source.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let rgba: [u8; 4] = pixel.2.0;
        let grayscale = (f32::from(rgba[0]) * 0.2126 
            + f32::from(rgba[1]) * 0.7152 
            + f32::from(rgba[2]) * 0.0722)
        .round()
        .clamp(0.0, 255.0) as i16;

        // corrected grayscale will probably be adding the rgb values then averaging them (TODO)
        let corrected_grayscale = (grayscale + current_errors[x as usize]).clamp(0, 255);
        // pushing them back into the octree to get another value (TODO)
        let bw_color: u8 = if corrected_grayscale <= 127 { 0 } else { 255 };

        // representing the error as an Rgb<u64> (TODO)
        let color_error = corrected_grayscale - i16::from(bw_color);

        // https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html
        // https://www.youtube.com/watch?v=ico4fJfohMQ
        let forward_x = (x + 1).clamp(0, image_width) as usize;
        let behind_x = if x <= 0 { 0 } else { (x - 1).clamp(0, image_width) as usize };
        current_errors[forward_x] += color_error * 2 / 4;
        forward_errors[behind_x] += color_error / 4;
        forward_errors[x] += color_error / 4;
        
        destination.put_pixel(x as u32, y as u32, Rgba([bw_color, bw_color, bw_color, rgba[3]]));

        if x >= image_width - 1 {
            current_errors.clone_from_slice(&forward_errors);
            forward_errors.fill(0);
        }
    }
}

// full color.
pub fn color_diff(lhs: &Rgb<u8>, rhs: &Rgb<u8>) -> u32 {
    let delta_r = lhs.0[0] as i32 - rhs.0[0] as i32;
    let delta_g = lhs.0[1] as i32 - rhs.0[1] as i32;
    let delta_b = lhs.0[2] as i32 - rhs.0[2] as i32;

    (3 * delta_r * delta_r + 6 * delta_g * delta_g + delta_b * delta_b) as u32
}

// low hanging optimizations: 
// - in place modification of rgb color.
const DITHER_COEF_DIVIDER: i16 = 16;
const COLOR_DIFF_THRESHOLD: u32 = 10;
fn dither_apply_error(err_color: &Rgb<i16>, color: &Rgb<u8>) -> Rgb<u8> {
    let [err_r, err_g, err_b] = err_color.0;
    let [src_r, src_g, src_b] = color.0.map(|c| i16::from(c));

    let r = src_r + (err_r / DITHER_COEF_DIVIDER);
    let g = src_g + (err_g / DITHER_COEF_DIVIDER);
    let b = src_b + (err_b / DITHER_COEF_DIVIDER);

    let dest_r = r.max(r).min(u8::MAX.into()) as u8;
    let dest_g = g.max(g).min(u8::MAX.into()) as u8;
    let dest_b = b.max(b).min(u8::MAX.into()) as u8;

    let rgb = Rgb([dest_r, dest_g, dest_b]);

    rgb
}

fn nearest_color_from_palette(palette: &Vec<Rgb<u8>>, rgb: &Rgb<u8>) -> usize {
    let mut smallest_diff = u32::MAX;
    let mut best_index: usize = 0;
    for (i, palette_rgb) in palette.iter().enumerate() {
        let diff = color_diff(palette_rgb, rgb);
        if diff < COLOR_DIFF_THRESHOLD {
            return i;
        }
        if diff < smallest_diff {
            smallest_diff = diff;
            best_index = i;
        }
    }

    best_index
}

// pre-allocated error_vec.
fn diffuse_error(error_vec: &mut Vec<Rgb<i16>>, width: usize, x: usize, y: usize, src_color: &Rgb<u8>, corrected_color: &Rgb<u8>) {
    let [src_r, src_g, src_b] = src_color.0;
    let [corr_r, corr_g, corr_b] = corrected_color.0;
    let error_index = (width * y) + x;
    let next_row_error_index = (width * (y + 1)) + x;

    let r_error = src_r as i16 - corr_r as i16;
    let g_error = src_g as i16 - corr_g as i16;
    let b_error = src_b as i16 - corr_b as i16;

    // sierra lite.
    if error_index + 1 <= error_vec.len() - 1 {
        add_colors(&mut error_vec[error_index + 1], &Rgb::<i16>([r_error * 2 / 4, g_error * 2 / 4, b_error * 2 / 4]));
    }
    if next_row_error_index + 1 <= error_vec.len() - 1 {
        add_colors(&mut error_vec[next_row_error_index + 1], &Rgb::<i16>([r_error / 4, g_error / 4, b_error / 4]));
    }
    if next_row_error_index <= error_vec.len() - 1 {
        add_colors(&mut error_vec[next_row_error_index], &Rgb::<i16>([r_error / 4, g_error / 4, b_error / 4]));
    }
}

fn sierra_lite_full_color(octree: &Octree, palette: &Vec<Rgb<u8>>, source: &DynamicImage, destination: &mut RgbImage) {
    let image_width = source.width() as usize;
    let mut error_vec = vec![Rgb::<i16>([0, 0, 0]); (source.width() * source.height()) as usize];

    for (x, y, rgba) in source.pixels() {
        let rgb = rgba.to_rgb();
        let index = octree.get_palette_index(rgb);
        let quantized_color = palette[index.unwrap()];
        destination.put_pixel(x as u32, y as u32, quantized_color);
    }

    for (x, y, rgba) in source.pixels() {
        let rgb = rgba.to_rgb();
        // - apply error
        let error_index = (image_width * y as usize) + x as usize;
        let dither_rgb = error_vec[error_index];
        let corrected_rgb = dither_apply_error(&dither_rgb, &rgb);
        // - get nearest color from palette (using color_diff func)
        let palette_index = octree.get_palette_index(corrected_rgb).expect("Octree on dither palette_index couldn't find a color!");
        // let palette_index = nearest_color_from_palette(palette, &corrected_rgb);
        // - diffuse error
        let palette_color = palette[palette_index];
        diffuse_error(&mut error_vec, image_width, x as usize, y as usize, &corrected_rgb, &palette_color);

        destination.put_pixel(x as u32, y as u32, palette_color);
    }
}

fn add_to_filename(path: &Path, addition: &str) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();

    let new_filename = format!("{}{}{}", stem, addition, extension);
    parent.join(new_filename)
}


fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut opts = Options::new(args.iter().map(String::as_str));
    let mut source_path: Option<&Path> = None;
    let mut color_size: Option<i32> = Some(256);

    while let Some(arg) = opts.next_arg().expect("Parsing error.") {
        match arg {
            Arg::Short('i') | Arg::Long("input") => {
                let opt = opts.value();
                match opt {
                    Ok(s) => source_path.replace(Path::new(s)),
                    Err(e) => panic!("{}", e)
                };
            }
            Arg::Short('c') | Arg::Long("color") => {
                let opt = opts.value();
                match opt {
                    Ok(s) => {
                        let res = s.parse::<i32>();
                        match res {
                            Ok(res) =>
                                if res < 8 {
                                    color_size.replace(res);
                                } else {
                                    panic!("Color is below 8!");
                                }
                            Err(_) => panic!("Color is not a number!"),
                        };
                    }
                    Err(e) => panic!("{}", e)
                };
            }
            Arg::Positional(_) | Arg::Short(_) | Arg::Long(_) => {}
        }
    }
    let source_path = source_path.expect("Set your input man.");
    let dest_path = add_to_filename(source_path, "_quant_dither");
    let absolute_source_path = path::absolute(source_path).unwrap().into_os_string().into_string().unwrap();
    let absolute_dest_path = path::absolute(dest_path).unwrap().into_os_string().into_string().unwrap();

    let img = image::open(absolute_source_path).expect("Can't find the file specified.");
    let mut new_img = image::RgbImage::new(img.width(), img.height());

    let mut recursive_octree = Octree::new();

    let start = Instant::now();
    for (_, _, rgba) in img.pixels() {
        recursive_octree.add_color(rgba.to_rgb());
    }

    println!("\nseconds to initialize: {:?}", Instant::now() - start);
    println!("Tree leaves count before quantization: {} color/s", recursive_octree.get_leaf_nodes().len());

    let palette = recursive_octree.make_palette(color_size.expect("Set the color size."));
    println!("Tree leaves count after quantization: {} color/s", recursive_octree.get_leaf_nodes().len());


    let start = Instant::now();

    sierra_lite_full_color(&recursive_octree, &palette, &img, &mut new_img);

    let duration = start.elapsed();
    println!("Image quantization took: {:?}", duration);
    println!("Time per pixel: {:.6} ms", duration.as_secs_f64() / (new_img.width() * new_img.height()) as f64 * 1000.0);
    println!("Pixels: {}", new_img.width() * new_img.height());

    if let Err(err) = new_img.save(absolute_dest_path) {
        println!("Image Save Error: {}", err);
    }
}

