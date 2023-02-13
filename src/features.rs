use crate::{typing::ImageData, model::Image, keypoints::KeyPoint, algorithm::Padding};

impl<T: ImageData> Image<T> {
    pub fn harris_corners(&self, threshold: f32, non_maximum_suppresion_dist: f32) -> Vec<KeyPoint> {
        const K: f32 = 0.05;

        // Ensure input format
        let cpy = self.clone().grayscale().to_single_channel();

        // Compute x and y gradients
        let mut x = cpy.to_f32();
        let mut y = cpy.to_f32();

        x.convolution(1, 1, &[1., 0., -1., 2., 0., -2., 1., 0., -1.], Padding::Repeat);
        y.convolution(1, 1, &[1., 2., 1., 0., 0., 0., -1., -2., -1.], Padding::Repeat);
        
        // Compute Harris response
        let mut xx = x.clone();
        let mut xy = x.clone();
        let mut yy = y.clone();

        xx.data.iter_mut().for_each(|i| *i *= *i);
        yy.data.iter_mut().for_each(|i| *i *= *i);
        xy.data.iter_mut().zip(y.data.iter()).for_each(|(i, j)| *i *= j);

        xx.gaussian_blur(1, 1.0, Padding::Repeat);
        xy.gaussian_blur(1, 1.0, Padding::Repeat);
        yy.gaussian_blur(1, 1.0, Padding::Repeat);

        xx.data.iter_mut().zip(xy.data.iter()).zip(yy.data.iter()).for_each(|((xx, xy), yy)| {
            let det = (*xx * yy) - (xy * xy);
            let trace = *xx + yy;

            *xx = det - (K * trace * trace);
        });

        xx.normalize();
        
        // Get keypoints
        let mut res = vec!();

        for i in 0..self.height {
            for j in 0..self.width {
                let r = xx.get_pixel_mut(j, i)[0];

                if r > threshold {
                    res.push(KeyPoint::new(j as f32, i as f32, (0, 255, 0), crate::keypoints::KeyPointShape::Cross));
                }
            }   
        }

        if non_maximum_suppresion_dist > 0.0 {
            res = res.iter().filter(|p| {
                let curr = xx.get_pixel_mut(p.x as usize, p.y as usize)[0];

                res.iter().filter(|p2| p.manhattan_distance(p2) < non_maximum_suppresion_dist)
                          .map(|p| xx.get_pixel_mut(p.x as usize, p.y as usize)[0])
                          .all(|v| v <= curr)

            }).cloned().collect();
        }

        return res;
    }

    fn fast_neighborhood(&self, i: usize, j: usize) -> [u8; 16] {
        return [
            self.get_pixel(j, i - 3)[0].to_u8(),
            self.get_pixel(j + 1, i - 3)[0].to_u8(),

            self.get_pixel(j + 2, i - 2)[0].to_u8(),

            self.get_pixel(j + 3, i - 1)[0].to_u8(),
            self.get_pixel(j + 3, i)[0].to_u8(),
            self.get_pixel(j + 3, i + 1)[0].to_u8(),

            self.get_pixel(j + 2, i + 2)[0].to_u8(),

            self.get_pixel(j + 1, i + 3)[0].to_u8(),
            self.get_pixel(j, i + 3)[0].to_u8(),
            self.get_pixel(j - 1, i + 3)[0].to_u8(),

            self.get_pixel(j - 2, i + 2)[0].to_u8(),

            self.get_pixel(j - 3, i + 1)[0].to_u8(),
            self.get_pixel(j - 3, i)[0].to_u8(),
            self.get_pixel(j - 3, i - 1)[0].to_u8(),

            self.get_pixel(j - 2, i - 2)[0].to_u8(),

            self.get_pixel(j - 1, i - 3)[0].to_u8(),
        ]
    }

    fn fast_neighborhood_fast_check(&self, i: usize, j: usize, p_up: i16, p_down: i16) -> bool {
        let p0 = self.get_pixel(j, i - 3)[0].to_u8() as i16;
        let p1 = self.get_pixel(j + 3, i)[0].to_u8() as i16;
        let p2 = self.get_pixel(j, i + 3)[0].to_u8() as i16;
        let p3 = self.get_pixel(j - 3, i)[0].to_u8() as i16;

        return (p0 > p_up) as u8 + (p1 > p_up) as u8 + (p2 > p_up) as u8 + (p3 > p_up) as u8 >= 3 ||
               (p0 < p_down) as u8 + (p1 < p_down) as u8 + (p2 < p_down) as u8 + (p3 < p_down) as u8 >= 3;
    }

    fn map_fast_pixel_value(p: u8, p_up: i16, p_down: i16) -> u8 {
        return if (p as i16) > p_up {
            0
        } else if (p as i16) < p_down {
            1
        } else {
            2
        }
    }

    pub fn fast_neighborhood_full_check(&self, i: usize, j: usize, p_up: i16, p_down: i16) -> bool {
        let n = self.fast_neighborhood(i, j);
        let mut consecutives = vec!();
        consecutives.reserve(16);

        n.into_iter()
        .map(|i| Image::<T>::map_fast_pixel_value(i, p_up, p_down))
        .for_each(|i| {
            if let Some((elem, its)) = consecutives.last_mut() {
                if *elem == i {
                    *its += 1;
                
                } else {
                    consecutives.push((i, 1));
                }

            } else {
                consecutives.push((i, 1));
            }
        });
        
        return consecutives.iter().any(|&i| i.1 >= 12) || 
                (
                    Image::<T>::map_fast_pixel_value(n[0], p_up, p_down) == Image::<T>::map_fast_pixel_value(n[15], p_up, p_down) && 
                    consecutives[0].1 + consecutives.last().unwrap().1 >= 12
                );
    }

    pub fn fast_score(&self, i: usize, j: usize) -> i32 {
        let p = self.get_pixel(j, i)[0].to_u8() as i32;
        return self.fast_neighborhood(i, j).into_iter().map(|i| p - i as i32).sum();
    }

    pub fn fast(&self, t: i16, non_maximum_suppresion_dist: f32) -> Vec<KeyPoint> {
        let cpy = self.clone().grayscale().to_single_channel();

        let mut res = vec!();

        for i in 3..cpy.height - 3 {
            for j in 3..cpy.width - 3 {
                let p = cpy.get_pixel(j, i)[0].to_u8() as i16;
                let p_up = p + t;
                let p_down = p - t;
                
                if cpy.fast_neighborhood_fast_check(i, j, p_up, p_down) && cpy.fast_neighborhood_full_check(i, j, p_up, p_down) {
                    res.push(KeyPoint::new(j as f32, i as f32, (0, 255, 0), crate::keypoints::KeyPointShape::Cross));
                }
            }
        }

        if non_maximum_suppresion_dist > 0.0 {
            res = res.iter().filter(|p| {
                let curr = cpy.fast_score(p.y as usize, p.x as usize);

                res.iter().filter(|p2| p.manhattan_distance(p2) < non_maximum_suppresion_dist)
                          .map(|p| cpy.fast_score(p.y as usize, p.x as usize))
                          .all(|v| v <= curr)

            }).cloned().collect();
        }

        return res;
    }
} 