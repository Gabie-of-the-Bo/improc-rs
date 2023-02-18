use crate::{typing::ImageData, model::Image};

#[derive(Clone, PartialEq)]
pub enum KeyPointShape {
    Dot, BigDot, Cross, Square
}

#[derive(Clone, PartialEq, Default)]
pub enum Descriptor {
    #[default]
    None,
    BRIEF([u8; 64]),
    RBRIEF([u8; 64])
}

impl Descriptor {
    pub fn distance(&self, b: &Descriptor) -> u32 {
        return match (self, b) {
            // Hamming distance
            (Descriptor::BRIEF(a), Descriptor::BRIEF(b)) |
            (Descriptor::RBRIEF(a), Descriptor::RBRIEF(b)) => a.iter().zip(b).map(|(i, j)| (i ^ j).count_ones()).sum(),

            _ => unreachable!()
        }
    }

    pub fn inner_array(&self) -> &[u8; 64] {
        return match self {
            // Hamming distance
            Descriptor::BRIEF(a) | Descriptor::RBRIEF(a) => &a,
            _ => unreachable!()
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct KeyPoint {
    pub x: f32,
    pub y: f32,

    pub descriptor: Descriptor,
    pub octave: usize,
    pub score: i32,
    pub angle: f32,

    pub color: (u8, u8, u8),
    shape: KeyPointShape
}

impl KeyPoint {
    pub fn new(x: f32, y: f32, color: (u8, u8, u8), shape: KeyPointShape) -> KeyPoint {
        return KeyPoint {x, y, color, shape, octave: 1, score: 0, angle: 0.0, descriptor: Descriptor::None};
    }

    pub fn manhattan_distance(&self, b: &KeyPoint) -> f32 {
        return (self.x - b.x).abs() + (self.y - b.y).abs();
    }

    pub fn get_shape_points(&self) -> Vec<(i32, i32)> {
        let xi = self.x as i32;
        let yi = self.y as i32;

        return match self.shape {
            KeyPointShape::Dot => vec!((xi, yi)),

            KeyPointShape::BigDot => vec!(
                (xi - 1, yi - 1), (xi - 1, yi), (xi - 1, yi + 1),
                (xi, yi - 1), (xi, yi), (xi, yi + 1),
                (xi + 1, yi - 1), (xi + 1, yi), (xi + 1, yi + 1),
            ),

            KeyPointShape::Cross => vec!(
                (xi, yi), 
                (xi - 1, yi - 1), (xi + 1, yi + 1), 
                (xi - 1, yi + 1), (xi + 1, yi - 1),
                (xi - 2, yi - 2), (xi + 2, yi + 2), 
                (xi - 2, yi + 2), (xi + 2, yi - 2)
            ),

            KeyPointShape::Square => vec!(
                (xi - 2, yi - 2), (xi - 2, yi - 1), (xi - 2, yi), (xi - 2, yi + 1), (xi - 2, yi + 2),
                (xi + 2, yi - 2), (xi + 2, yi - 1), (xi + 2, yi), (xi + 2, yi + 1), (xi + 2, yi + 2),
                (xi - 1, yi + 2), (xi, yi + 2), (xi + 1, yi + 2),
                (xi - 1, yi - 2), (xi, yi - 2), (xi + 1, yi - 2)
            )
        }
    }

    pub fn draw<T: ImageData>(&self, image: &mut Image<T>) {
        for (j, i) in self.get_shape_points() {
            if j >= 0 && i >= 0 && j < image.width as i32 && i < image.height as i32 {
                let p = image.get_pixel_mut(j as usize, i as usize);
    
                p[0] = T::from_u8(self.color.0);
                p[1] = T::from_u8(self.color.1);
                p[2] = T::from_u8(self.color.2);
            }
        }
    }
}