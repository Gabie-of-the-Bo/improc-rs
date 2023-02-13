use improc::{model::Image, algorithm::Padding, colors::Gradient, keypoints::KeyPoint, timing::time_many};

pub fn main() {
    let mut img = Image::read("room2.png");

    let keypoints = img.fast(10, 5.0);

    println!("{:?}", time_many(|| {
        img.fast(10, 3.0);
    }, 100));

    println!("Corners: {}", keypoints.len());

    keypoints.iter().for_each(|p| p.draw(&mut img));

    img.show("Borders");
}