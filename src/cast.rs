use crate::{model::Image, typing::ImageData};

impl<T: ImageData> Image<T> {
    pub fn to_u8(&self) -> Image<u8> {
        return Image { 
            height: self.height, 
            width: self.width, 
            channels: self.channels, 
            color: self.color, 
            data: self.data.iter().copied().map(T::to_u8).collect()
        }
    }

    pub fn to_f32(&self) -> Image<f32> {
        return Image { 
            height: self.height, 
            width: self.width, 
            channels: self.channels, 
            color: self.color, 
            data: self.data.iter().copied().map(T::to_f32).collect()
        }
    }

    pub fn to_bool(&self) -> Image<bool> {
        return Image { 
            height: self.height, 
            width: self.width, 
            channels: self.channels, 
            color: self.color, 
            data: self.data.iter().copied().map(T::to_bool).collect()
        }
    }
}