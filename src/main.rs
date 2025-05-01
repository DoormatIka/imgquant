
pub mod core;
pub mod tests;

use image::{ColorType, DynamicImage, GenericImage, GenericImageView, Pixel, Rgba};
use std::{env, path::{self, Path, PathBuf}, time::Instant, u32, u8};
use getargs::{Arg, Options};
use thiserror::Error;

use core::rgb_helpers::IRgb;
use core::accum_octree::LeafOctree;

enum DitherMode {
    Base,
    FloydSteinberg,
    SierraLite,
}

// low hanging optimizations: 
// - in place modification of rgb color.
// - there are too many copies in this entire codebase.
fn apply_error_with_dither(err_color: IRgb<i16>, color: IRgb<u8>) -> IRgb<u8> {
    let src_color = IRgb::<i16>::safe_cast(color).unwrap();
    let mut color = src_color + err_color;
    color.clamp(0, u8::MAX.into());
    IRgb::<u8>::safe_cast(color).unwrap()
}

// expensive function to account for the octree not covering all colors!
// dithering makes new colors out of nowhere due to errors + original color = new color
fn nearest_color_from_palette(palette: &Vec<IRgb<u8>>, rgb: IRgb<u8>) -> usize {
    let mut smallest_diff = u32::MAX;
    let mut best_index: usize = 0;
    for (i, palette_rgb) in palette.iter().enumerate() {
        let diff = palette_rgb.color_diff(rgb);
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

fn diffuse_pixel_sierra_lite(error_vec: &mut Vec<IRgb<i16>>, rgb: IRgb<u8>, error_index: usize, next_row_error_index: usize) {
    let next_row_error_index = next_row_error_index.max(0).min(error_vec.len() - 1);
    let front_curr_row = (error_index + 1).max(0).min(error_vec.len() - 1);
    let front_next_row = (next_row_error_index + 1).max(0).min(error_vec.len() - 1);

    error_vec[front_curr_row] += IRgb::safe_cast(rgb).unwrap() * 2 / 4;
    error_vec[front_next_row] += IRgb::safe_cast(rgb).unwrap() / 4;
    error_vec[next_row_error_index] += IRgb::safe_cast(rgb).unwrap() / 4;
}
fn diffuse_pixel_floyd_steinberg(error_vec: &mut Vec<IRgb<i16>>, rgb: IRgb<u8>, error_index: usize, next_row_error_index: usize) {
    let front_curr_row = (error_index + 1).max(0).min(error_vec.len() - 1);
    let behind_next_row = (next_row_error_index - 1).max(0).min(error_vec.len() - 1);
    let next_row_error_index = next_row_error_index.max(0).min(error_vec.len() - 1);
    let front_next_row = (next_row_error_index + 1).max(0).min(error_vec.len() - 1);

    error_vec[front_curr_row] += IRgb::safe_cast(rgb).unwrap() * 7 / 16;
    error_vec[front_next_row] += IRgb::safe_cast(rgb).unwrap() / 16;
    error_vec[next_row_error_index] += IRgb::safe_cast(rgb).unwrap() * 5 / 16;
    error_vec[behind_next_row] += IRgb::safe_cast(rgb).unwrap() * 3 / 16;
}

// pre-allocated error_vec.
fn diffuse_error(error_vec: &mut Vec<IRgb<i16>>, width: usize, x: usize, y: usize, src_color: IRgb<u8>, corrected_color: IRgb<u8>, dither_mode: &DitherMode) {
    let error_index = (width * y) + x;
    let next_row_error_index = (width * (y + 1)) + x;
    let error_color = src_color - corrected_color;

    match dither_mode {
        DitherMode::SierraLite => diffuse_pixel_sierra_lite(error_vec, error_color, error_index, next_row_error_index),
        DitherMode::FloydSteinberg => diffuse_pixel_floyd_steinberg(error_vec, error_color, error_index, next_row_error_index),
        DitherMode::Base => panic!("base!!"),
    }

    error_vec[error_index] = IRgb::from_array([0, 0, 0]);
}

fn base_quantize(octree: &LeafOctree, palette: &Vec<IRgb<u8>>, source: &DynamicImage, destination: &mut DynamicImage) {
    let (width, height) = source.dimensions();
    for y in 0..height {
        for x in 0..width {
            let rgba = source.get_pixel(x, y);
            let rgb = rgba.to_rgb();
            let palette_index = octree.get_palette_index(IRgb(rgb), true).expect("LeafOctree on dither palette_index couldn't find a color!");
            let palette_color = palette[palette_index];

            destination.put_pixel(x as u32, y as u32, Rgba([palette_color.0[0], palette_color.0[1], palette_color.0[2], rgba.0[3]]));
        }
    }
}

fn quantize_dither_image(octree: &LeafOctree, palette: &Vec<IRgb<u8>>, source: &DynamicImage, destination: &mut DynamicImage, dither_mode: &DitherMode) {
    let image_width = source.width() as usize;
    let mut error_vec = vec![IRgb::<i16>::from_array([0, 0, 0]); (source.width() * source.height()) as usize];

    let (width, height) = source.dimensions();
    for y in 0..height {
        for x in 0..width {
            let rgba = source.get_pixel(x, y);
            let rgb = IRgb(rgba.to_rgb());
            let error_index = (image_width * y as usize) + x as usize;
            let dither_rgb = error_vec[error_index];
            let corrected_rgb = apply_error_with_dither(dither_rgb, rgb);

            let palette_index = match octree.get_palette_index(corrected_rgb, true) {
                Some(index) => index,
                None => nearest_color_from_palette(palette, rgb),
            };
            let palette_color = palette[palette_index];
            diffuse_error(&mut error_vec, image_width, x as usize, y as usize, corrected_rgb, palette_color, &dither_mode);

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

fn print_palette(palette: &Vec<IRgb<u8>>) {
    print!("Palette: ");
    for rgb in palette.iter() {
        print_color_box(rgb);
        print!("\x1B[0m");
    }
    println!("\x1B[0m");
}

fn print_color_box(rgb: &IRgb<u8>) {
    let [r, g, b] = rgb.get_inner().0;
    print!("\x1B[48;2;{};{};{}m ", r, g, b);
}

struct ParsedOptions {
    source_path: Box<Path>,
    color_size: i32,
    dither_mode: DitherMode,
    depth: usize,
}

#[derive(Error, Debug)]
enum ParseErrors {
    #[error(r#"Early return for help."#)]
    Help,
    #[error(r#"Unknown option: {0}"#)]
    UnknownOption(String),
    #[error(r#"An argument for "{0}" is missing."#)]
    MissingArgument(String),
    #[error(r#"Invalid argument: {0}"#)]
    InvalidArgument(String),
}

fn parse_cli() -> Result<ParsedOptions, ParseErrors> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut opts = Options::new(args.iter().map(String::as_str));
    let mut source_path: Option<Box<Path>> = None;
    let mut color_size = 256;
    let mut dither_mode = DitherMode::FloydSteinberg;
    let mut depth: usize = 6;
    let mut option_count = 0;

    while let Some(arg) = opts.next_arg().expect("Parsing error.") {
        match arg {
            Arg::Short('h') | Arg::Long("help") => {
                return Err(ParseErrors::Help);
            },
            Arg::Long("dither") => {
                let opt = opts.value();
                match opt {
                    Ok(s) => match s.to_lowercase().as_str() {
                        "base" => dither_mode = DitherMode::Base,
                        "sierralite" | "sl" => dither_mode = DitherMode::SierraLite,
                        "floydsteinberg" | "fs" => dither_mode = DitherMode::FloydSteinberg,
                        _ => return Err(ParseErrors::InvalidArgument(format!("{} is not a valid dither mode. Options: base, sierralite, floydsteinberg", s))),
                    }
                    Err(_) => return Err(ParseErrors::MissingArgument("dither".to_string()))
                };
            }
            Arg::Short('i') | Arg::Long("input") => {
                let opt = opts.value();
                match opt {
                    Ok(s) => {
                        let buf = PathBuf::from(s).into_boxed_path();
                        source_path.replace(buf);
                    },
                    Err(_) => return Err(ParseErrors::MissingArgument("input".to_string()))
                }
            }
            Arg::Short('d') | Arg::Long("depth") => { // unreachable.
                let opt = opts.value();
                match opt {
                    Ok(s) => {
                        let res = s.parse::<usize>();
                        match res {
                            Ok(d) => {
                                println!("{}", d);
                                if d <= 10 && d > 2 {
                                    depth = d;
                                } else {
                                    return Err(ParseErrors::InvalidArgument("Depth must be more than 2 and less than or equal to 10.".to_string()))
                                }
                            },
                            Err(_) => return Err(ParseErrors::InvalidArgument("Depth is not a number.".to_string())),
                        };
                    }
                    Err(_) => return Err(ParseErrors::MissingArgument("depth".to_string()))
                }
            }
            Arg::Short('c') | Arg::Long("color") => {
                let opt = opts.value();
                match opt {
                    Ok(s) => {
                        let res = s.parse::<i32>();
                        match res {
                            Ok(res) =>
                                if res >= 2 {
                                    color_size = res;
                                } else {
                                    return Err(ParseErrors::InvalidArgument(format!("Color size {} is below 2.", color_size)))
                                }
                            Err(_) => return Err(ParseErrors::InvalidArgument("Color size is not a number.".to_string())),
                        };
                    }
                    Err(_) => return Err(ParseErrors::MissingArgument("color".to_string()))
                };
            }
            Arg::Positional(l) | Arg::Long(l) => return Err(ParseErrors::UnknownOption(l.to_string())),
            Arg::Short(s) => return Err(ParseErrors::UnknownOption(s.to_string())),
        }
        option_count += 1;
    };

    if option_count <= 0 {
        return Err(ParseErrors::Help);
    }

    if let Some(source_path) = source_path {
        Ok(ParsedOptions { source_path, color_size, dither_mode, depth })
    } else {
        Err(ParseErrors::MissingArgument("source path".to_string()))
    }
}

fn run_quantization_pipeline(opts: ParsedOptions) {
    let ParsedOptions { source_path, color_size, dither_mode, depth } = opts;

    let dest_path = add_to_filename(&source_path, "_quant_dither");
    let absolute_source_path = path::absolute(&source_path).unwrap().into_os_string().into_string().unwrap();
    let absolute_dest_path = path::absolute(&dest_path).unwrap().into_os_string().into_string().unwrap();

    let img = match image::open(absolute_source_path) {
        Ok(img) => img,
        Err(err) => return println!("FileError: {}", err),
    };
    let file_name = source_path.file_name().unwrap().to_str().unwrap();
    let (image_width, image_height) = img.dimensions();
    let image_color = img.color();
    print!(r#"
filename: {}
width, height: ({}, {})
color type: {:?}, bits per pixel: {}, channel count: {}
    "#, file_name, image_width, image_height, image_color, image_color.bits_per_pixel(), image_color.channel_count());
    let mut new_img = DynamicImage::new(image_width, image_height, image_color);
    let mut recursive_octree = LeafOctree::new(depth);

    let start = Instant::now();
    for (_, _, rgba) in img.pixels() {
        recursive_octree.add_color(IRgb(rgba.to_rgb()));
    }

    println!("\nseconds to initialize: {:?}", Instant::now() - start);
    println!("tree leaves count before quantization: {} color/s", recursive_octree.get_leaf_nodes().len());

    let palette = recursive_octree.make_palette(color_size);
    println!("tree leaves count after quantization: {} color/s", recursive_octree.get_leaf_nodes().len());

    print_palette(&palette);

    let start = Instant::now();
    match dither_mode {
        DitherMode::Base => base_quantize(&recursive_octree, &palette, &img, &mut new_img),
        DitherMode::SierraLite | DitherMode::FloydSteinberg => quantize_dither_image(&recursive_octree, &palette, &img, &mut new_img, &dither_mode),
    };
    let duration = start.elapsed();
    println!("image quantization took: {:?}", duration);
    println!("time per pixel: {:.6} ms", duration.as_secs_f64() / (new_img.width() * new_img.height()) as f64 * 1000.0);
    println!("pixels: {}", new_img.width() * new_img.height());

    let dest_img = match image_color {
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
        _ => return println!("Unsupported color type!"),
    };

    if let Err(err) = dest_img.save(absolute_dest_path) {
        println!("Image Save Error: {}", err);
    }
}

fn main() {
    let parsed_options = parse_cli();
    match parsed_options {
        Ok(opts) => run_quantization_pipeline(opts),
        Err(err) => match err {
            ParseErrors::Help => {
                println!(
                    r#"Usage: imgquant [-h] [-vvvv]
    A fast simple image quantizer.

    Options:
        -h, --help     help
        -i, --input    file to quantize
        -d, --depth    octree depth (2 to 8)
        -c, --color    number of colors in the octree.
        --dither       modes for dithering [base, sierralite, floydsteinberg]
                    "#
                    );
            },
            err => println!("{}", err),
        }
    }
}

