
pub mod core;

use core::octree::{add_colors, div_colors, mul_colors, Octree};
use std::{fs::File, path::{self, Path}, time::{Duration, Instant}};
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgb, RgbImage, Rgba, RgbaImage};

fn grayscale(source: &DynamicImage, destination: &mut RgbaImage) {
    for pixel in source.pixels() {
        let x = pixel.0;
        let y = pixel.1;
        let rgba: [u8; 4] = pixel.2.0;
        let grayscale = (f32::from(rgba[0]) * 0.2126 
            + f32::from(rgba[1]) * 0.7152 
            + f32::from(rgba[2]) * 0.0722)
        .round()
        .clamp(0.0, 255.0) as u8;

        destination.put_pixel(x, y, Rgba([grayscale, grayscale, grayscale, rgba[3]]));
    }
}

fn bw_quant(source: &DynamicImage, destination: &mut RgbaImage) {
    for pixel in source.pixels() {
        let x = pixel.0;
        let y = pixel.1;
        let rgba: [u8; 4] = pixel.2.0;
        let grayscale = (f32::from(rgba[0]) * 0.2126 
            + f32::from(rgba[1]) * 0.7152 
            + f32::from(rgba[2]) * 0.0722)
        .round()
        .clamp(0.0, 255.0) as u8;
        let bw_color = if grayscale <= 127 { 0 } else { 255 };

        destination.put_pixel(x, y, Rgba([bw_color, bw_color, bw_color, rgba[3]]));
    }
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

fn sierra_lite(source: &DynamicImage, destination: &mut RgbaImage) {
    let image_width = source.width() as usize;

    let mut current_errors = vec![0i16; image_width + 1];
    let mut forward_errors = vec![0i16; image_width + 1];

    for pixel in source.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let rgba: [u8; 4] = pixel.2.0;
        // TODO:
        //
        // quant color using the octree instead.
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

fn dither_apply_error(pixel_coords: (usize, usize), color: &Rgb<u8>, corrected_color: &Rgb<u8>) {
    let col: u32 = 0;

}

fn sierra_lite_full_color(octree: &Octree, palette: &Vec<Rgb<u8>>, source: &DynamicImage, destination: &mut RgbImage) {
    let image_width = source.width() as usize;

    let mut current_errors = vec![Rgb::<u8>([0, 0, 0]); image_width + 1];
    let mut forward_errors = vec![Rgb::<u8>([0, 0, 0]); image_width + 1];

    for pixel in source.pixels() {
        let x = pixel.0 as usize;
        let y = pixel.1 as usize;
        let rgb = pixel.2.to_rgb();
        // TODO:
        // quant color using the octree instead.
        let corrected_color = Rgb([
            rgb.0[0].saturating_add(current_errors[x as usize].0[0]),
            rgb.0[1].saturating_add(current_errors[x as usize].0[1]),
            rgb.0[2].saturating_add(current_errors[x as usize].0[2]),
        ]);
        let index = octree.get_palette_index(corrected_color);
        let quantized_color = palette[index.unwrap()];
        // representing the error as an Rgb<u64> (TODO)

        let color_error = Rgb([
            rgb.0[0].saturating_sub(quantized_color.0[0]),
            rgb.0[1].saturating_sub(quantized_color.0[1]),
            rgb.0[2].saturating_sub(quantized_color.0[2]),
        ]);
        // https://tannerhelland.com/2012/12/28/dithering-eleven-algorithms-source-code.html
        // https://www.youtube.com/watch?v=ico4fJfohMQ
        let forward_x = (x + 1).clamp(0, image_width) as usize;
        let behind_x = if x <= 0 { 0 } else { (x - 1).clamp(0, image_width) as usize };
        add_colors(&mut current_errors[forward_x], &div_colors(&mut mul_colors(&color_error, &Rgb([2, 2, 2])), &Rgb([4, 4, 4])));
        // current_errors[forward_x] += color_error * 2 / 4;
        add_colors(&mut forward_errors[behind_x], &div_colors(&color_error, &Rgb([4, 4, 4])));
        // forward_errors[behind_x] += color_error / 4;
        add_colors(&mut forward_errors[x], &div_colors(&color_error, &Rgb([4, 4, 4])));
        // forward_errors[x] += color_error / 4;
        
        destination.put_pixel(x as u32, y as u32, quantized_color);

        if x >= image_width - 1 {
            current_errors.clone_from_slice(&forward_errors);
            forward_errors.fill(Rgb([0, 0, 0]));
        }
    }
}

/*
fn main() {
    let source_path = Path::new("images/sakuya_gardening.png");
    let absolute_source_path = path::absolute(source_path).unwrap().into_os_string().into_string().unwrap();
    let img = image::open(absolute_source_path).unwrap();
    let mut new_img = image::RgbImage::new(img.width(), img.height());

    bw_quant_line_filter(&img, &mut new_img);

    let source_path = Path::new("images/sakuya_line_filter.png");
    let absolute_source_path = path::absolute(source_path).unwrap().into_os_string().into_string().unwrap();
    new_img.save(absolute_source_path);
}
*/

fn main() {
    let source_path = Path::new("images/Portal_Companion_Cube.png");
    let absolute_source_path = path::absolute(source_path).unwrap().into_os_string().into_string().unwrap();
    let img = image::open(absolute_source_path).unwrap();

    let mut recursive_octree = Octree::new();

    let start = Instant::now();
    for (_, _, rgba) in img.pixels() {
        recursive_octree.add_color(rgba.to_rgb());
    }

    println!("\nseconds to initialize: {:?}", Instant::now() - start);
    println!("Tree leaves length: {}", recursive_octree.get_leaf_nodes().len());

    let palette = recursive_octree.make_palette(256);
    println!("Tree leaves after quant length: {}", recursive_octree.get_leaf_nodes().len());

    let mut new_img = image::RgbImage::new(img.width(), img.height());

    let start = Instant::now();

    sierra_lite_full_color(&recursive_octree, &palette, &img, &mut new_img);

    let duration = start.elapsed();
    println!("Quantization took: {:?}", duration);
    println!("Time per pixel: {:.6} ms", duration.as_secs_f64() / (new_img.width() * new_img.height()) as f64 * 1000.0);
    println!("Pixels: {}", new_img.width() * new_img.height());

    let source_path = Path::new("images/Portal_Companion_Cube_sierra_lite_dither.png");
    let absolute_source_path = path::absolute(source_path).unwrap().into_os_string().into_string().unwrap();
    if let Err(err) = new_img.save(absolute_source_path) {
        println!("Image Save Error: {}", err);
    }
}

