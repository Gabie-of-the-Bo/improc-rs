use crunchy::unroll;
use kiddo::KdTree;
use lazy_static::lazy_static;

use crate::{typing::ImageData, model::{Image, ColorSpace}, keypoints::{KeyPoint, Descriptor}, algorithm::{Padding, Interpolation}, data::BRIEF_OFFSETS};

lazy_static!(
    static ref FAST_PATCH_SIZE: i32 = 32;
    static ref HALF_FAST_PATCH_SIZE: i32 = *FAST_PATCH_SIZE / 2;
    static ref CIRCLE_LIM: Vec<i32> = (0..=*FAST_PATCH_SIZE).map(
        |i| (((i as f32 - *HALF_FAST_PATCH_SIZE as f32) / *HALF_FAST_PATCH_SIZE as f32).acos().sin() * *HALF_FAST_PATCH_SIZE as f32) as i32
    ).collect();
);

impl<T: ImageData> Image<T> {
    fn non_maximum_suppression_kd(&self, mut keypoints: Vec<KeyPoint>, mut non_maximum_suppression_dist: f32) -> Vec<KeyPoint> {
        non_maximum_suppression_dist *= non_maximum_suppression_dist;

        let mut tree = KdTree::new();
        keypoints.iter().for_each(|p| tree.add(&[p.x, p.y], p.clone()).unwrap());

        keypoints.retain(|p| {
            tree.within_unsorted(&[p.x, p.y], non_maximum_suppression_dist, &kiddo::distance::squared_euclidean)
                .unwrap().iter()
                .all(|(_, i)| i.score <= p.score)
        });

        return keypoints;
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
                    let mut kp = KeyPoint::new(j as f32, i as f32, (0, 255, 0), crate::keypoints::KeyPointShape::Cross);
                    kp.score = self.get_pixel(j, i)[0].to_u8() as i32;

                    res.push(kp);
                }
            }   
        }

        if non_maximum_suppression_dist > 0.0 {
            res = self.non_maximum_suppression_kd(res, non_maximum_suppression_dist);
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
                    let mut kp = KeyPoint::new(j as f32, i as f32, (0, 255, 0), crate::keypoints::KeyPointShape::Square);
                    kp.score = self.fast_score(i, j);

                    res.push(kp);
                }
            }
        }

        if non_maximum_suppression_dist > 0.0 {
            res = self.non_maximum_suppression_kd(res, non_maximum_suppression_dist);
        }

        return res;
    }

    pub fn compute_angle(&self, kp: &mut KeyPoint) {
        let xi = kp.x as usize;
        let yi = kp.y as usize;

        let mut mx = 0;
        let mut my = 0;

        for i in -*HALF_FAST_PATCH_SIZE..=*HALF_FAST_PATCH_SIZE {
            let circle_limit = CIRCLE_LIM[(i + *HALF_FAST_PATCH_SIZE) as usize];
            let mut sum = 0;

            for j in -circle_limit..=circle_limit {
                let p = self.get_pixel((xi as i32 + j) as usize, (yi as i32 + i) as usize)[0].to_u8() as i32;
                mx += j * p;    
                sum += p;
            }

            my += i * sum;
        }

        kp.angle = (my as f32).atan2(mx as f32);
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
        let mut cpy = self.clone();

        if self.color != ColorSpace::Gray {
            cpy.grayscale();
        }

        if self.channels != 1 {
            cpy = cpy.to_single_channel();
        }

        cpy.gaussian_blur(1, 1.0, Padding::Repeat);
        let mut pyramid = vec!(cpy);

        for _ in 0..3 {
            let mut new_level = pyramid.last().unwrap().clone();
            new_level.resize(new_level.width / 2, new_level.height / 2, Interpolation::Nearest);
            new_level.gaussian_blur(1, 1.0, Padding::Repeat);
            pyramid.push(new_level);
        }

        let mut keypoints = vec!();

        for (i, p) in pyramid.iter().enumerate() {
            let mut level_keypoints = p.fast(t, 0.0, 46);
            p.rotated_brief(&mut level_keypoints);

            let scale_x = self.width as f32 / p.width as f32;
            let scale_y = self.height as f32 / p.height as f32;

            keypoints.extend(level_keypoints.into_iter().map(|mut p| {
                p.octave = i;
                p.x *= scale_x;
                p.y *= scale_y;
                p
            }));
        }

        keypoints = self.non_maximum_suppression_kd(keypoints, non_maximum_suppression_dist);

        return keypoints;
    }
} 