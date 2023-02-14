use improc::{model::Image, timing::time_many};

pub fn main() {
    let mut img = Image::read("room3.png");

    println!("{:?}", time_many(|| { img.orb(15, 5.0); }, 100));

    let keypoints = img.orb(10, 5.0);
    keypoints.iter().for_each(|p| p.draw(&mut img));

    println!("Corners: {}", keypoints.len());

    img.show("Borders");
}