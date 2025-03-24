
pub mod core;

use core::{octree::Octree, octree_flat::FlatOctree};
use std::{fs::File, path::{self, Path}, time::{Duration, Instant}};
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgb, Rgba, RgbaImage};

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

fn bw_quant_line_filter(source: &DynamicImage, destination: &mut RgbaImage) {
    let mut color_error: i16 = 0;
    let image_width = source.width();
    
    for (x, y, pixel) in source.pixels() {
        let rgba: [u8; 4] = pixel.0;
        let grayscale = (f32::from(rgba[0]) * 0.2126 
            + f32::from(rgba[1]) * 0.7152 
            + f32::from(rgba[2]) * 0.0722)
        .round()
        .clamp(0.0, 255.0) as i16;

        let corrected_grayscale = (grayscale - color_error).clamp(0, 255);

        let bw_color: u8 = if corrected_grayscale <= 127 - grayscale {
            0
        } else {
            255
        };
        color_error = corrected_grayscale + i16::from(bw_color);

        destination.put_pixel(x, y, Rgba([bw_color, bw_color, bw_color, rgba[3]]));

        if x > image_width {
            // color_error = 0;
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

fn img_main() {
    let source_path = Path::new("images/sakuya_gardening.png");
    let processed_path = Path::new("images/sakuya_line_filter.png");

    let absolute_source_path = path::absolute(source_path).unwrap().into_os_string().into_string().unwrap();
    let absolute_processed_path = path::absolute(processed_path).unwrap().into_os_string().into_string().unwrap();

    let img = image::open(absolute_source_path).unwrap();
    let mut output = RgbaImage::new(img.width(), img.height());

    bw_quant_line_filter(&img, &mut output);

    let _ = File::create(absolute_processed_path.clone()).unwrap();
    match output.save(absolute_processed_path) {
        Ok(()) => println!("Suwako image processed."),
        Err(err) => println!("{}", err),
    }
}


fn main() {
    let source_path = Path::new("images/sakuya_gardening.png");
    let absolute_source_path = path::absolute(source_path).unwrap().into_os_string().into_string().unwrap();
    let img = image::open(absolute_source_path).unwrap();

    // let mut flat_octree = FlatOctree::new();
    let mut recursive_octree = Octree::new();

    let mut colors: Vec<Rgb<u8>> = vec![];
    for (_, _, rgba) in img.pixels() {
        colors.push(rgba.to_rgb());
    }

    let start = Instant::now();
    for color in colors {
        recursive_octree.add_color(color);
    }

    println!("\nseconds: {:?}", Instant::now() - start);
    println!("Tree leaves length: {}", recursive_octree.get_leaf_nodes().len());

    let palette = recursive_octree.make_palette(256);
    println!("Tree leaves after quant length: {}", recursive_octree.get_leaf_nodes().len());

    println!("Palette: {:?}", palette);

    let mut new_img = image::RgbImage::new(img.width(), img.height());
    for x in 0..new_img.width() {
        for y in 0..new_img.height() {
            let pixel = img.get_pixel(x, y).to_rgb();
            let palette_index = recursive_octree.get_palette_index(pixel);
            new_img.put_pixel(x, y, palette[palette_index]);
        }
    }

    let source_path = Path::new("images/sakuya_gardening_quantized.png");
    let absolute_source_path = path::absolute(source_path).unwrap().into_os_string().into_string().unwrap();
    new_img.save(absolute_source_path);
}


