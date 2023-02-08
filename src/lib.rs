pub mod model;
pub mod typing;
pub mod cast;
pub mod algorithm;
pub mod noise;
pub mod metrics;
pub mod timing;
mod utils;

#[cfg(test)]
mod tests {
    use crate::{metrics::mse, model::Image, algorithm::Padding};

    #[test]
    fn salt_and_pepper_median() {
        let mut img = Image::read("img.png");

        let original = img.clone();
        img.salt_and_pepper(25);

        let mut fixed = img.clone();
        fixed.median_filter(3, Padding::Repeat);

        let err_orig_noise = mse(&original, &img);
        let err_orig_fixed = mse(&original, &fixed);

        assert!(err_orig_noise > err_orig_fixed);
        assert!(err_orig_fixed < 1e-3);
    }

    #[test]
    fn color_conversions() {
        let img = Image::read("img.png");
        let mut transformed = img.clone();

        transformed.hsl().rgb();

        let err = mse(&img, &transformed);

        assert!(err < 1e-3);
    }
}
