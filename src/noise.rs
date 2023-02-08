use crate::{typing::ImageData, model::Image};

use rand::Rng;

impl<T: ImageData> Image<T> {
    pub fn salt_and_pepper(&mut self, percentage: u8) -> &mut Self {
        assert!(percentage <= 100);

        let mut rng = rand::thread_rng();

        self.for_each_pixel_mut(|p| {
            for c in p.iter_mut() {
                if rng.gen_ratio(percentage as u32, 100) {
                    *c = if rng.gen_ratio(1, 2) { T::min() } else { T::max() };
                }
            }
        });

        return self;
    }
}