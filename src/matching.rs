use acap::{vp::VpTree, Proximity, NearestNeighbors};
use crate::{keypoints::{KeyPoint}, model::Image, typing::ImageData};

impl Proximity for &KeyPoint {
    type Distance = i32;

    fn distance(&self, other: &Self) -> Self::Distance {
        self.descriptor.distance(&other.descriptor) as i32
    }
}

pub fn match_descriptors<'v>(from: &'v Vec<KeyPoint>, to: &'v Vec<KeyPoint>, ratio_test: f32) -> Vec<(&'v KeyPoint, &'v KeyPoint)> {
    let tree = VpTree::from_iter(to.iter());
    
    let nn = from.iter().map(|p| (p, tree.k_nearest(&p, 2)))
                        .filter(|(_, v)| v.len() == 2)
                        .filter(|(_, v)| (v[0].distance as f32 / v[1].distance as f32) < ratio_test)
                        .map(|(a, v)| (a, *v[0].item))
                        .collect::<Vec<_>>();

    return nn;
}

pub fn draw_match<T: ImageData>(stacked_image: &mut Image<T>, kps: (&KeyPoint, &KeyPoint)) {
    stacked_image.line(
        kps.0.x, kps.0.y,
        kps.1.x + (stacked_image.width / 2) as f32, kps.1.y,
        kps.0.color
    );

    kps.0.draw(stacked_image);

    let mut cpy = kps.1.clone();
    cpy.x += (stacked_image.width / 2) as f32;

    cpy.draw(stacked_image);    
}

pub fn visualize_matches<T: ImageData>(a: &Image<T>, b: &Image<T>, matches: &Vec<(&KeyPoint, &KeyPoint)>) {
    let mut stacked_image = a.horizontal_stack(b);

    matches.iter().for_each(|m| draw_match(&mut stacked_image, *m));

    stacked_image.show("Matches");
}