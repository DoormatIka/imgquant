
use image::{GenericImage, GenericImageView, ImageReader, Pixel, Rgba, RgbaImage};

fn main() {
    let img = ImageReader::open("../images/sakuya_gardening.png")
        .unwrap()
        .decode()
        .unwrap();
    let mut output = RgbaImage::new(img.width(), img.height());

    for pixel in img.pixels() {
        let x = pixel.0;
        let y = pixel.1;
        let rgba: Rgba<u8> = pixel.2;
        output.put_pixel(x, y, rgba);
    }
}
