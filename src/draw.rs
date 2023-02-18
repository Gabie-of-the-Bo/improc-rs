use crate::{typing::ImageData, model::Image};

impl<T: ImageData> Image<T> {
    // Slow algorithm because it relies on float operations, but good enough for our purposes (debug)
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: (u8, u8, u8)) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let slope = dy / dx;

        if dx > dy {
            for j in (x0.min(x1) as usize)..=(x0.max(x1) as usize) {
                let i = y0 + slope * (j as f32 - x0);

                let p = self.get_pixel_mut(j, i as usize);
                p[0] = T::from_u8(color.0);
                p[1] = T::from_u8(color.1);
                p[2] = T::from_u8(color.2);
            }

        } else {
            let slope = 1.0 / slope;

            for i in (y0.min(y1) as usize)..=(y0.max(y1) as usize) {
                let j = x0 + slope * (i as f32 - y0);

                let p = self.get_pixel_mut(j as usize, i);
                p[0] = T::from_u8(color.0);
                p[1] = T::from_u8(color.1);
                p[2] = T::from_u8(color.2);
            }
        }
    }
}