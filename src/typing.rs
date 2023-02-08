use std::fmt::Debug;

pub trait ImageData: Copy + PartialOrd + Debug {
    fn to_u8(self) -> u8;
    fn to_f32(self) -> f32;
    fn to_bool(self) -> bool;

    fn from_u8(val: u8) -> Self;
    fn from_f32(val: f32) -> Self;
    fn from_bool(val: bool) -> Self;

    fn min() -> Self;
    fn max() -> Self;
}

const INV_255: f32 = 1.0 / 255.0;

impl ImageData for u8 {
    fn to_u8(self) -> u8 {
        return self;
    }

    fn to_f32(self) -> f32 {
        return self as f32 * INV_255;
    }

    fn to_bool(self) -> bool {
        return self != 0;
    }

    fn from_u8(val: u8) -> Self {
        return val;
    }

    fn from_f32(val: f32) -> Self {
        return val.to_u8();
    }

    fn from_bool(val: bool) -> Self {
        return val.to_u8();
    }

    fn min() -> Self {
        return 0;
    }

    fn max() -> Self {
        return 255;
    }
}

impl ImageData for f32 {
    fn to_u8(self) -> u8 {
        return (self * 255.0) as u8;
    }

    fn to_f32(self) -> f32 {
        return self;
    }

    fn to_bool(self) -> bool {
        return self != 0.0;
    }

    fn from_u8(val: u8) -> Self {
        return val.to_f32();
    }

    fn from_f32(val: f32) -> Self {
        return val;
    }

    fn from_bool(val: bool) -> Self {
        return val.to_f32();
    }

    fn min() -> Self {
        return 0.0;
    }

    fn max() -> Self {
        return 1.0;
    }
}

impl ImageData for bool {
    fn to_u8(self) -> u8 {
        return self as u8 * 255;
    }

    fn to_f32(self) -> f32 {
        return self as u8 as f32;
    }

    fn to_bool(self) -> bool {
        return self;
    }

    fn from_u8(val: u8) -> Self {
        return val.to_bool();
    }

    fn from_f32(val: f32) -> Self {
        return val.to_bool();
    }

    fn from_bool(val: bool) -> Self {
        return val;
    }

    fn min() -> Self {
        return false;
    }

    fn max() -> Self {
        return true;
    }
}