use improc::{model::Image, algorithm::Padding};

pub fn main() {
    let img = Image::read("img.png")
        .grayscale()
        .to_single_channel()
        .gaussian_blur(2, 0.5, Padding::Repeat)
        .threshold(100)
        .sobel();

    img.show("Borders");
}