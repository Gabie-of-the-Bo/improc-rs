use crunchy::unroll;

use crate::{typing::ImageData, model::{Image, ColorSpace}, keypoints::{KeyPoint, Descriptor}, algorithm::Padding, data::BRIEF_OFFSETS};

impl<T: ImageData> Image<T> {
    fn non_maximum_suppression_kd<F: Fn(&KeyPoint) -> i32>(&self, keypoints: Vec<KeyPoint>, non_maximum_suppression_dist: f32, metric: F) -> Vec<KeyPoint> {
        let tree = kd_tree::KdTree::build_by_ordered_float(keypoints);

        return tree.into_iter().filter(|&p| {
            let curr = metric(p);
            tree.within_radius(p, non_maximum_suppression_dist).iter()
                .all(|p| curr >= metric(p))
                
        }).cloned().collect::<Vec<KeyPoint>>();
    }

    pub fn harris_corners(&self, threshold: f32, non_maximum_suppression_dist: f32) -> Vec<KeyPoint> {
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

        if non_maximum_suppression_dist > 0.0 {
            res = self.non_maximum_suppression_kd(res, non_maximum_suppression_dist, |p| {
                self.get_pixel(p.x as usize, p.y as usize)[0].to_u8() as i32
            });
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

    // 0 -> lower, 1 -> none, 2 -> higher
    fn map_fast_pixel_value(p: u8, p_up: i16, p_down: i16) -> u8 {
        return 1 + ((p as i16) > p_up) as u8 - ((p as i16) < p_down) as u8;
    }

    fn fast_neighborhood_full_check(&self, i: usize, j: usize, p_up: i16, p_down: i16) -> bool {
        let mut n = self.fast_neighborhood(i, j);
        let mut consecutives = vec!();
        consecutives.reserve(8);

        n[0] = Image::<T>::map_fast_pixel_value(n[0], p_up, p_down);
        consecutives.push((n[0], 1));

        unroll! {
            for idx in 1..16 {
                n[idx] = Image::<T>::map_fast_pixel_value(n[idx], p_up, p_down);
                let (elem, run) = consecutives.last_mut().unwrap();
    
                if *elem == n[idx] {
                    *run += 1;
                
                } else {
                    consecutives.push((n[idx], 1));
                }
            }
        }

        return consecutives.iter().any(|&i| i.0 != 1 && i.1 >= 12) || 
               (n[0] != 1 && n[0] == n[15] && consecutives[0].1 + consecutives.last().unwrap().1 >= 12);
    }

    fn fast_score(&self, i: usize, j: usize) -> i32 {
        let p = self.get_pixel(j, i)[0].to_u8() as i32;
        return self.fast_neighborhood(i, j).into_iter().map(|i| p - i as i32).sum();
    }

    pub fn fast(&self, t: i16, non_maximum_suppression_dist: f32, mut margin: usize) -> Vec<KeyPoint> {
        assert!(self.channels == 1 && self.color == ColorSpace::Gray);

        let mut res = vec!();
        margin = margin.max(3);

        for i in margin..self.height - margin {
            for j in margin..self.width - margin {
                let p = self.get_pixel(j, i)[0].to_u8() as i16;
                let p_up = p + t;
                let p_down = p - t;
                
                if self.fast_neighborhood_fast_check(i, j, p_up, p_down) && self.fast_neighborhood_full_check(i, j, p_up, p_down) {
                    res.push(KeyPoint::new(j as f32, i as f32, (0, 255, 0), crate::keypoints::KeyPointShape::Cross));
                }
            }
        }

        if non_maximum_suppression_dist > 0.0 {
            res = self.non_maximum_suppression_kd(res, non_maximum_suppression_dist, |p| {
                self.fast_score(p.y as usize, p.x as usize)
            });
        }

        return res;
    }

    pub fn compute_angle(&self, kp: &mut KeyPoint) {
        let xi = kp.x as usize;
        let yi = kp.y as usize;

        let p00 = self.get_pixel(xi - 1, yi - 1)[0].to_f32();
        let p01 = self.get_pixel(xi, yi - 1)[0].to_f32();
        let p02 = self.get_pixel(xi + 1, yi - 1)[0].to_f32();
        let p10 = self.get_pixel(xi - 1, yi)[0].to_f32();
        let p12 = self.get_pixel(xi + 1, yi)[0].to_f32();
        let p20 = self.get_pixel(xi - 1, yi + 1)[0].to_f32();
        let p21 = self.get_pixel(xi, yi + 1)[0].to_f32();
        let p22 = self.get_pixel(xi + 1, yi + 1)[0].to_f32();

        let mx = p02 + p12 + p22 - p00 - p10 - p20;
        let my = p20 + p21 + p22 - p00 - p01 - p02;

        kp.angle = my.atan2(mx);
    }

    fn compute_brief(&self, kp: &mut KeyPoint) {
        let mut res = [0u8; 64];
        let xi = kp.x as i32;
        let yi = kp.y as i32;

        for i in 0..64 {
            let idx = i * 8;

            unroll! {
                for j in 0..8 {
                    let [x0, y0, x1, y1] = BRIEF_OFFSETS[idx + j];
                    let p0 = self.get_pixel((xi + x0 as i32) as usize, (yi + y0 as i32) as usize)[0];
                    let p1 = self.get_pixel((xi + x1 as i32) as usize, (yi + y1 as i32) as usize)[0];
                    res[i] |= ((p0 < p1) as u8) << j;
                }
            }
        }

        kp.descriptor = Descriptor::BRIEF(res);
    }

    pub fn brief(&self, keypoints: &mut Vec<KeyPoint>) {
        assert!(self.channels == 1 && self.color == ColorSpace::Gray);

        for kp in keypoints {
            self.compute_brief(kp);
        }
    }

    fn compute_rotated_brief(&self, kp: &mut KeyPoint) {
        let mut res = [0u8; 64];
        let xi = kp.x as i32;
        let yi = kp.y as i32;

        let s = kp.angle.sin();
        let c = kp.angle.cos();

        for i in 0..64 {
            let idx = i * 8;

            unroll! {
                for j in 0..8 {
                    let [x0, y0, x1, y1] = BRIEF_OFFSETS[idx + j];

                    let x0 = (x0 as f32 * c - y0 as f32 * s) as i32;
                    let x1 = (x1 as f32 * c - y1 as f32 * s) as i32;
                    let y0 = (y0 as f32 * c + x0 as f32 * s) as i32;
                    let y1 = (y1 as f32 * c + x1 as f32 * s) as i32;

                    let p0 = self.get_pixel((xi + x0) as usize, (yi + y0) as usize)[0];
                    let p1 = self.get_pixel((xi + x1) as usize, (yi + y1) as usize)[0];
                    res[i] |= ((p0 < p1) as u8) << j;
                }
            }
        }

        kp.descriptor = Descriptor::RBRIEF(res);
    }

    pub fn rotated_brief(&self, keypoints: &mut Vec<KeyPoint>) {
        assert!(self.channels == 1 && self.color == ColorSpace::Gray);

        for kp in keypoints {
            self.compute_angle(kp);
            self.compute_rotated_brief(kp);
        }
    }

    pub fn orb(&self, t: i16, non_maximum_suppression_dist: f32) -> Vec<KeyPoint> {
        let correct_format = self.channels == 1 && self.color == ColorSpace::Gray;
        let opt;

        let cpy = if correct_format { 
            self
        
        } else {            
            opt = Some(self.clone().grayscale().to_single_channel());
            opt.as_ref().unwrap()
        };

        let mut keypoints = cpy.fast(t, non_maximum_suppression_dist, 46);
        cpy.rotated_brief(&mut keypoints);

        return keypoints;
    }
} 