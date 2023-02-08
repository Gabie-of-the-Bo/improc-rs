use crate::{typing::ImageData, model::Image};

pub fn mse<T: ImageData>(a: &Image<T>, b: &Image<T>) -> f32 {
    assert!(a.channels == b.channels && a.width == b.width && a.height == b.height);
    
    return a.data.iter().copied().map(T::to_f32)
                 .zip(b.data.iter().copied().map(T::to_f32))
                 .map(|(a, b)| (a - b).powf(2.0))
                 .sum::<f32>() / a.data.len() as f32;
}