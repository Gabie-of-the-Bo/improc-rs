use improc::{model::Image, matching::{match_descriptors, visualize_matches}};

pub fn main() {
    let img1 = Image::read("houses/img1.png");
    let keypoints1 = img1.orb(20, 5.0);
    
    let img2 = Image::read("houses/img2.png");
    let keypoints2 = img2.orb(20, 5.0);

    println!("Found {} and {} features", keypoints1.len(), keypoints2.len());

    let matches = match_descriptors(&keypoints1, &keypoints2, 0.8);

    println!("{} matches found", matches.len());

    visualize_matches(&img1, &img2, &matches);
}