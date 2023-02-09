use crate::{model::{Image, ColorSpace}, typing::ImageData, colors::Gradient};

#[derive(Copy, Clone)]
pub enum Padding {
    Zeros, Repeat
}

impl<T: ImageData> Image<T> {
    pub fn to_single_channel(&mut self) -> Image<T> {
        assert!(self.color == ColorSpace::Gray);

        let mut res = Image::<T>::zeros(self.height, self.width, 1);
        res.data.iter_mut().zip(self.data.chunks_exact(self.channels)).for_each(|(a, b)| *a = b[0]);
        res.color = ColorSpace::Gray;

        return res;
    }

    pub fn to_three_channels(&mut self) -> Image<T> {
        assert!(self.channels == 1);

        let mut res = Image::<T>::zeros(self.height, self.width, 3);

        res.pixels().zip(self.data.iter()).for_each(|(a, b)| {
            a[0] = *b;
            a[1] = *b;
            a[2] = *b;
        });

        return res;
    }

    fn rgb_to_grayscale(&mut self) -> &mut Self {
        assert!(self.channels == 3 && self.color == ColorSpace::RGB);

        self.for_each_pixel_mut(|p| {
            let g = p[0].to_f32() * 0.299 + 
                    p[1].to_f32() * 0.587 + 
                    p[2].to_f32() * 0.114;

            let tg = T::from_f32(g);

            p[0] = tg;
            p[1] = tg;
            p[2] = tg;
        });

        self.color = ColorSpace::Gray;

        return self;
    }

    fn rgb_to_hsl(&mut self) -> &mut Self {
        assert!(self.channels == 3 && self.color == ColorSpace::RGB);

        self.for_each_pixel_mut(|p| {
            let r = p[0].to_f32();
            let g = p[1].to_f32();
            let b = p[2].to_f32();

            let min = r.min(g).min(b);
            let max = r.max(g).max(b);
            let c = max - min;
            let l = min + c / 2.0;
            let s = (max - l) / l.min(1.0 - l);
    
            let h = if c == 0.0 { 0.0 } 
               else if max == r { 60.0 * ((g - b) / c) }
               else if max == g { 60.0 * (2.0 + (b - r) / c) }
               else             { 60.0 * (4.0 + (r - g) / c) };
    
            p[0] = T::from_f32(h / 360.0);
            p[1] = T::from_f32(s);
            p[2] = T::from_f32(l);
        });

        self.color = ColorSpace::HSL;

        return self;
    }

    fn hsl_to_rgb(&mut self) -> &mut Self {
        assert!(self.channels == 3 && self.color == ColorSpace::HSL);

        self.for_each_pixel_mut(|p| {
            let h = p[0].to_f32() * 6.0;
            let s = p[1].to_f32();
            let l = p[2].to_f32();

            let c = (1.0 - (l + l - 1.0).abs()) * s;
            let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
            let m = l - (c / 2.0);

            let (r, g, b) = if h <= 1.0 { (c, x, 0.0) }
                       else if h <= 2.0 { (x, c, 0.0) }
                       else if h <= 3.0 { (x, c, 0.0) }
                       else if h <= 4.0 { (x, c, 0.0) }
                       else if h <= 5.0 { (x, c, 0.0) }
                       else             { (x, c, 0.0) };

            p[0] = T::from_f32(r + m);
            p[1] = T::from_f32(g + m);
            p[2] = T::from_f32(b + m);
        });

        self.color = ColorSpace::RGB;

        return self;
    }

    fn hsl_to_gray(&mut self) -> &mut Self {
        assert!(self.channels == 3 && self.color == ColorSpace::HSL);

        let mut res = Image::<T>::zeros(self.height, self.width, 1);
        res.data.iter_mut().zip(self.data.chunks_exact(self.channels)).for_each(|(a, b)| *a = b[2]);
        res.color = ColorSpace::Gray;

        *self = res;

        return self;
    }

    pub fn fake_color(&self, gradient: &Gradient) -> Image<u8> {
        assert!(self.channels == 1 && self.color == ColorSpace::Gray);

        let mut res = Image::zeros(self.height, self.width, 3);

        res.pixels().zip(self.data.iter()).for_each(|(p, d)| {
            let (r, g, b) = gradient.get_color(d.to_f32());
            
            p[0] = r;
            p[1] = g;
            p[2] = b;
        });

        return  res;
    }

    pub fn grayscale(&mut self) -> &mut Self {
        return match self.color {
            ColorSpace::Gray => self,
            ColorSpace::RGB => self.rgb_to_grayscale(),
            ColorSpace::HSL => self.hsl_to_gray()
        };
    }

    pub fn hsl(&mut self) -> &mut Self {
        return match self.color {
            ColorSpace::Gray =>  {
                if self.channels == 1 {
                    *self = self.to_three_channels();
                }

                self.rgb_to_hsl()
            },
            ColorSpace::RGB => self.rgb_to_hsl(),
            ColorSpace::HSL => self
        };
    }

    pub fn rgb(&mut self) -> &mut Self {
        return match self.color {
            ColorSpace::Gray => unimplemented!("Unable to recreate color from grayscale. Use to_three_channels or fake_color instead"),
            ColorSpace::RGB => self,
            ColorSpace::HSL => self.hsl_to_rgb()
        };
    }

    pub fn threshold(&mut self, thres: T) -> Image<bool> {
        assert!(self.channels == 1);

        return Image {
            height: self.height,
            width: self.width,
            channels: 1,
            color: ColorSpace::Gray,
            data: self.data.iter().map(|i| *i > thres).collect()
        };
    }

    pub fn sliding_window<F>(&mut self, im: i32, jm: i32, height: i32, width: i32, padding: Padding, mut f: F) where F: FnMut(i32, i32, i32, i32, &mut [T]) {
        for i in (im - height)..(im + height + 1) {
            for j in (jm - width)..(jm + width + 1) {
                let wi = i + height - im;
                let wj = j + width - jm;

                if i >= 0 && j >= 0 && i < self.height as i32 && j < self.width as i32 {
                    f(wi, wj, i, j, self.get_pixel_mut(j as usize, i as usize));

                } else {
                    match padding {
                        Padding::Zeros => {
                            f(wi, wj, i, j, &mut vec!(T::min(); self.channels));
                        }

                        Padding::Repeat => {
                            f(wi, wj, i, j, self.get_pixel_mut(
                                j.clamp(0, self.width as i32 - 1) as usize, 
                                i.clamp(0, self.height as i32 - 1) as usize
                            ))
                        }
                    }
                }
            }
        }
    }

    pub fn convolution(&mut self, height: usize, width: usize, window: &[f32], padding: Padding) -> &mut Self {
        let side_w = width * 2 + 1;
        let side_h = height * 2 + 1;

        assert!(window.len() == side_h * side_w);
        
        let channels = self.channels;
        let mut cpy = self.clone();

        for i in 0..self.height {
            for j in 0..self.width {
                let mut val = vec!(0.0; self.channels);

                cpy.sliding_window(i as i32, j as i32, height as i32, width as i32, padding, |wi, wj, _, _, p| {
                    let w_idx = side_w * wi as usize + wj as usize;

                    for c in 0..channels {
                        val[c] += p[c].to_f32() * window[w_idx];
                    }
                });

                let px = self.get_pixel_mut(j, i);

                for c in 0..channels {
                    px[c] = T::from_f32(val[c]);
                }
            }
        }

        return self;
    }

    pub fn non_linear_filter<F: FnMut(&mut [T]) -> T>(&mut self, height: usize, width: usize, mut f: F, padding: Padding) -> &mut Self {
        let channels = self.channels;
        let mut cpy = self.clone();

        for i in 0..self.height {
            for j in 0..self.width {
                let mut pixels = vec!(vec!(); self.channels);

                cpy.sliding_window(i as i32, j as i32, height as i32, width as i32, padding, |_, _, _, _, p| {
                    for c in 0..channels {
                        pixels[c].push(p[c]);
                    }
                });

                let px = self.get_pixel_mut(j, i);

                for c in 0..channels {
                    px[c] = f(&mut pixels[c]);
                }
            }
        }

        return self;
    }

    pub fn median_filter(&mut self, window: usize, padding: Padding) -> &mut Self {
        return self.non_linear_filter(window, window, |p| {
            p.sort_by(|a, b| a.partial_cmp(b).unwrap());
            return p[p.len() / 2];
        }, padding);
    }

    pub fn blur(&mut self, window: usize, padding: Padding) -> &mut Self {
        let w_side = window * 2 + 1;
        let w_size = w_side * w_side;

        return self.convolution(window, window, &vec!(1.0 / w_size as f32; w_size), padding);
    }

    pub fn gaussian_blur(&mut self, window: usize, sigma: f32, padding: Padding) -> &mut Self {
        let w_side = window * 2 + 1;
        let w_size = w_side * w_side;

        // Compute gaussian kernel
        let mut kernel = vec!(0.0; w_size);
        let s2 = sigma.powf(2.0);
        let p = 1.0 / (2.0 * std::f32::consts::PI * s2);

        for i in 0..w_side {
            for j in 0..w_side {
                let k_idx = i * w_side + j;
                let i_n = i as i32 - window as i32;
                let j_n = j as i32 - window as i32;

                kernel[k_idx] = p * (-((i_n * i_n + j_n * j_n) as f32) / (s2 + s2)).exp();
            }   
        }

        // Normalize kernel
        let kernel_sum: f32 = kernel.iter().sum();
        kernel.iter_mut().for_each(|i| *i /= kernel_sum);

        return self.convolution(window, window, &kernel, padding);
    }

    pub fn sobel(&mut self) -> Image<f32> {
        assert!(self.channels == 1);

        // Compute x and y gradients
        let mut x = self.to_f32();
        let mut y = self.to_f32();

        x.convolution(1, 1, &[1., 2., 1., 0., 0., 0., -1., -2., -1.], Padding::Repeat);
        y.convolution(1, 1, &[1., 0., -1., 2., 0., -2., 1., 0., -1.], Padding::Repeat);

        // Compute gradient magnitude and normalize
        x.data.iter_mut().zip(y.data).for_each(|(a, b)| *a = ((*a * *a) + (b * b)).sqrt());
        x.normalize();

        x.color = ColorSpace::Gray;

        if x.channels > 1 {
            x = x.to_single_channel();
        }

        return x;
    }
}

impl Image<f32> {
    pub fn normalize(&mut self) {
        let max = self.data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().to_f32();
        self.data.iter_mut().for_each(|i| *i /= max);
    }
}