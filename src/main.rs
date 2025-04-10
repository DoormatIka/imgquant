
pub mod core;

use core::octree::{add_colors, div_colors, mul_colors, Octree};
use std::{cmp, env, fs::File, io::BufReader, path::{self, Path, PathBuf}, time::{Duration, Instant}, u32, u8};
use getargs::{Arg, Options};
use image::{error, ColorType, DynamicImage, GenericImage, GenericImageView, ImageBuffer, Luma, Pixel, Rgb, RgbImage, Rgba, RgbaImage};

enum DitherMode {
    Base,
    FloydSteinberg,
    SierraLite,
}

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
pub fn color_diff<L, R>(lhs: &Rgb<L>, rhs: &Rgb<R>) -> u32 
where 
    L: Copy,
    R: Copy,
    i32: From<L> + From<R>,
{
    let delta_r = i32::from(lhs.0[0]) - i32::from(rhs.0[0]);
    let delta_g = i32::from(lhs.0[1]) - i32::from(rhs.0[1]);
    let delta_b = i32::from(lhs.0[2]) - i32::from(rhs.0[2]);

    (3 * delta_r * delta_r + 6 * delta_g * delta_g + delta_b * delta_b) as u32
}

// low hanging optimizations: 
// - in place modification of rgb color.
fn dither_apply_error(err_color: &Rgb<i16>, color: &Rgb<u8>) -> Rgb<u8> {
    let [err_r, err_g, err_b] = err_color.0;
    let [src_r, src_g, src_b] = color.0.map(|c| i16::from(c));

    let r = src_r + err_r;
    let g = src_g + err_g;
    let b = src_b + err_b;

    let dest_r = r.max(0).min(u8::MAX.into()) as u8;
    let dest_g = g.max(0).min(u8::MAX.into()) as u8;
    let dest_b = b.max(0).min(u8::MAX.into()) as u8;

    let rgb = Rgb([dest_r, dest_g, dest_b]);

    rgb
}

// expensive function to account for the octree not covering all colors!
// dithering makes new colors out of nowhere due to errors + original color = new color
fn nearest_color_from_palette(palette: &Vec<Rgb<u8>>, rgb: &Rgb<u8>) -> usize {
    let mut smallest_diff = u32::MAX;
    let mut best_index: usize = 0;
    for (i, palette_rgb) in palette.iter().enumerate() {
        let diff = color_diff(palette_rgb, rgb);
        if diff < 10 {
            return i;
        }
        if diff < smallest_diff {
            smallest_diff = diff;
            best_index = i;
        }
    }

    best_index
}

fn diffuse_pixel_sierra_lite(error_vec: &mut Vec<Rgb<i16>>, r_error: i16, g_error: i16, b_error: i16, error_index: usize, next_row_error_index: usize) {
    let next_row_error_index = next_row_error_index.max(0).min(error_vec.len() - 1);
    let front_curr_row = (error_index + 1).max(0).min(error_vec.len() - 1);
    let front_next_row = (next_row_error_index + 1).max(0).min(error_vec.len() - 1);

    add_colors(&mut error_vec[front_curr_row], &Rgb::<i16>([r_error * 2 / 4, g_error * 2 / 4, b_error * 2 / 4]));
    add_colors(&mut error_vec[front_next_row], &Rgb::<i16>([r_error / 4, g_error / 4, b_error / 4]));
    add_colors(&mut error_vec[next_row_error_index], &Rgb::<i16>([r_error / 4, g_error / 4, b_error / 4]));
}
fn diffuse_pixel_floyd_steinberg(error_vec: &mut Vec<Rgb<i16>>, r_error: i16, g_error: i16, b_error: i16, error_index: usize, next_row_error_index: usize) {
    let front_curr_row = (error_index + 1).max(0).min(error_vec.len() - 1);
    let behind_next_row = (next_row_error_index - 1).max(0).min(error_vec.len() - 1);
    let next_row_error_index = next_row_error_index.max(0).min(error_vec.len() - 1);
    let front_next_row = (next_row_error_index + 1).max(0).min(error_vec.len() - 1);

    add_colors(&mut error_vec[front_curr_row], &Rgb::<i16>([r_error * 7 / 16, g_error * 7 / 16, b_error * 7 / 16]));
    add_colors(&mut error_vec[front_next_row], &Rgb::<i16>([r_error / 16, g_error / 16, b_error / 16]));
    add_colors(&mut error_vec[next_row_error_index], &Rgb::<i16>([r_error * 5 / 16, g_error * 5 / 16, b_error * 5 / 16]));
    add_colors(&mut error_vec[behind_next_row], &Rgb::<i16>([r_error * 3 / 16, g_error * 3 / 16, b_error * 3 / 16]));
}

// pre-allocated error_vec.
fn diffuse_error(error_vec: &mut Vec<Rgb<i16>>, width: usize, x: usize, y: usize, src_color: &Rgb<u8>, corrected_color: &Rgb<u8>, dither_mode: &DitherMode) {
    let [src_r, src_g, src_b] = src_color.0;
    let [corr_r, corr_g, corr_b] = corrected_color.0;
    let error_index = (width * y) + x;
    let next_row_error_index = (width * (y + 1)) + x;

    let r_error = src_r as i16 - corr_r as i16;
    let g_error = src_g as i16 - corr_g as i16;
    let b_error = src_b as i16 - corr_b as i16;

    match dither_mode {
        DitherMode::SierraLite => diffuse_pixel_sierra_lite(error_vec, r_error, g_error, b_error, error_index, next_row_error_index),
        DitherMode::FloydSteinberg => diffuse_pixel_floyd_steinberg(error_vec, r_error, g_error, b_error, error_index, next_row_error_index),
        DitherMode::Base => panic!("base!!"),
    }

    error_vec[error_index] = Rgb([0, 0, 0]);
}

fn octree_full_color(octree: &Octree, palette: &Vec<Rgb<u8>>, source: &DynamicImage, destination: &mut DynamicImage) {
    let (width, height) = source.dimensions();
    for y in 0..height {
        for x in 0..width {
            let rgba = source.get_pixel(x, y);
            let rgb = rgba.to_rgb();
            let palette_index = octree.get_palette_index(rgb, true).expect("Octree on dither palette_index couldn't find a color!");
            let palette_color = palette[palette_index];

            destination.put_pixel(x as u32, y as u32, Rgba([palette_color.0[0], palette_color.0[1], palette_color.0[2], rgba.0[3]]));
        }
    }
}

fn quantize_dither_image(octree: &Octree, palette: &Vec<Rgb<u8>>, source: &DynamicImage, destination: &mut DynamicImage, dither_mode: &DitherMode) {
    let image_width = source.width() as usize;
    let mut error_vec = vec![Rgb::<i16>([0, 0, 0]); (source.width() * source.height()) as usize];

    let (width, height) = source.dimensions();
    for y in 0..height {
        for x in 0..width {
            let rgba = source.get_pixel(x, y);
            let rgb = rgba.to_rgb();
            // - apply error
            let error_index = (image_width * y as usize) + x as usize;
            let dither_rgb = error_vec[error_index];
            let corrected_rgb = dither_apply_error(&dither_rgb, &rgb);
            // - get nearest color from palette
            let palette_index = match octree.get_palette_index(corrected_rgb, false) {
                Some(index) => index,
                None => nearest_color_from_palette(palette, &rgb)
            };
            let palette_color = palette[palette_index];
            // let palette_index = nearest_color_from_palette(palette, &corrected_rgb);
            // - diffuse error
            diffuse_error(&mut error_vec, image_width, x as usize, y as usize, &corrected_rgb, &palette_color, &dither_mode);

            destination.put_pixel(x as u32, y as u32, Rgba([palette_color.0[0], palette_color.0[1], palette_color.0[2], rgba.0[3]]));
        }
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
    let mut dither_mode = DitherMode::SierraLite;

    while let Some(arg) = opts.next_arg().expect("Parsing error.") {
        match arg {
            Arg::Short('d') | Arg::Long("dither") => {
                let opt = opts.value();
                match opt {
                    Ok(s) => match s.to_lowercase().as_str() {
                        "base" => dither_mode = DitherMode::Base,
                        "sierralite" | "sl" => dither_mode = DitherMode::SierraLite,
                        "floydsteinberg" | "fs" => dither_mode = DitherMode::FloydSteinberg,
                        _ => panic!(r#""{}" is not a valid option! [base, sierra]"#, s)
                    }
                    Err(e) => panic!("{}", e)
                };
            }
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
                                if res >= 2 {
                                    color_size.replace(res);
                                } else {
                                    panic!("Color is below 2!");
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
    println!("colortype: {:?}", img.color());
    let mut new_img = DynamicImage::new(img.width(), img.height(), img.color());
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

    match dither_mode {
        DitherMode::Base => octree_full_color(&recursive_octree, &palette, &img, &mut new_img),
        DitherMode::SierraLite | DitherMode::FloydSteinberg => quantize_dither_image(&recursive_octree, &palette, &img, &mut new_img, &dither_mode),
    };

    let duration = start.elapsed();
    println!("Image quantization took: {:?}", duration);
    println!("Time per pixel: {:.6} ms", duration.as_secs_f64() / (new_img.width() * new_img.height()) as f64 * 1000.0);
    println!("Pixels: {}", new_img.width() * new_img.height());

    let dest_img = match img.color() {
        ColorType::L8 => DynamicImage::ImageLuma8(new_img.to_luma8()),
        ColorType::L16 => DynamicImage::ImageLuma16(new_img.to_luma16()),
        ColorType::La8 => DynamicImage::ImageLumaA8(new_img.to_luma_alpha8()),
        ColorType::La16 => DynamicImage::ImageLumaA16(new_img.to_luma_alpha16()),
        ColorType::Rgb8 => DynamicImage::ImageRgb8(new_img.to_rgb8()),
        ColorType::Rgb16 => DynamicImage::ImageRgb16(new_img.to_rgb16()),
        ColorType::Rgb32F => DynamicImage::ImageRgb32F(new_img.to_rgb32f()),
        ColorType::Rgba8 => DynamicImage::ImageRgba8(new_img.to_rgba8()),
        ColorType::Rgba16 => DynamicImage::ImageRgba16(new_img.to_rgba16()),
        ColorType::Rgba32F => DynamicImage::ImageRgba32F(new_img.to_rgba32f()),
        _ => panic!("Unsupported color type!"),
    };

    if let Err(err) = dest_img.save(absolute_dest_path) {
        println!("Image Save Error: {}", err);
    }
}

