use improc::{model::Image, algorithm::Padding, colors::Gradient};

pub fn main() {
    let img = Image::read("img.png")
        .grayscale()
        .to_single_channel()
        .gaussian_blur(2, 0.5, Padding::Repeat)
        .threshold(100)
        .sobel()
        .gaussian_blur(5, 1.0, Padding::Repeat)
        .fake_color(&Gradient::simple(vec!((0, 0, 0), (255, 0, 0))));

    img.show("Borders");
}