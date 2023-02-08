use std::slice::ChunksExactMut;

use ::image::GenericImageView;
use pixels::{SurfaceTexture, Pixels};
use winit::{
    dpi::{LogicalSize},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder, event::{Event, VirtualKeyCode}, platform::run_return::EventLoopExtRunReturn,
};
use winit_input_helper::WinitInputHelper;

use crate::{utils::FPSCounter, typing::ImageData};

#[derive(Copy, Clone, PartialEq)]
pub enum ColorSpace {
    RGB, Gray, HSL
}

#[derive(Clone)]
pub struct Image<T: ImageData> {
    pub height: usize,
    pub width: usize,
    pub channels: usize,
    pub color: ColorSpace,

    pub data: Vec<T>
}

impl<T: ImageData> Image<T> {
    pub fn zeros(height: usize, width: usize, channels: usize) -> Self {
        return Image { height, width, channels, color: ColorSpace::RGB, data: vec!(T::min(); height * width * channels) }
    }

    pub fn ones(height: usize, width: usize, channels: usize) -> Self {
        return Image { height, width, channels, color: ColorSpace::RGB, data: vec!(T::max(); height * width * channels) }
    }

    pub fn get_pixel_mut(&mut self, x: usize, y: usize) -> &mut [T] {
        let idx = (self.width * y + x) * self.channels;
        return &mut self.data[idx..idx + self.channels];
    }

    pub fn pixels(&mut self) -> ChunksExactMut<T> {
        return self.data.chunks_exact_mut(self.channels);
    }

    pub fn for_each_pixel_mut<F: FnMut(&mut [T])>(&mut self, f: F) {
        self.pixels().for_each(f);
    }

    pub fn show(&self, title: &str) {
        self.animate(title, 30.0, |_| {});
    }

    pub fn animate<'r, F>(&self, title: &str, fps: f64, mut f: F) where F: FnMut(&mut Image<T>) {
        let mut event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();
    
        let window = {
            let size = LogicalSize::new(self.width as u32, self.height as u32);
            let scaled_size = LogicalSize::new(self.width as u32, self.height as u32);
            WindowBuilder::new()
                .with_title(title)
                .with_inner_size(scaled_size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);

        let mut pixels = Pixels::new(self.width as u32, self.height as u32, surface_texture).unwrap();
        let mut cpy = self.clone();
    
        let mut fps_counter = FPSCounter::new(fps);
        fps_counter.start();

        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::RedrawRequested(_) => {
                    if pixels.render().is_err()
                    {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }    
                }

                _ if input.update(&event) && fps_counter.update() => {
                    if input.key_held(VirtualKeyCode::Escape) || input.quit() {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    
                    match self.channels {
                        3 => {
                            // RGB
                            pixels.get_frame().chunks_exact_mut(4).zip(cpy.data.chunks_exact(3)).for_each(|(a, b)| {
                                a[0] = b[0].to_u8();
                                a[1] = b[1].to_u8();
                                a[2] = b[2].to_u8();
                                a[3] = 255;
                            });
                        }

                        1 => {
                            // Grayscale
                            pixels.get_frame().chunks_exact_mut(4).zip(&cpy.data).for_each(|(a, b)| {
                                a[0] = b.to_u8();
                                a[1] = b.to_u8();
                                a[2] = b.to_u8();
                                a[3] = 255;
                            });
                        }

                        _ => unimplemented!("Unsupported number of channels ({})", self.channels)
                    }

                    window.request_redraw();

                    f(&mut cpy);
                }

                _ => {}
            }
        });
    }
}

impl Image<u8> {
    pub fn read(path: &str) -> Self {
        let image = image::open(path).unwrap_or_else(|_| panic!("Unable to open image at {}", path));
        let (width, height) = image.dimensions();

        let mut res = Image::<u8>::zeros(height as usize, width as usize, 3);

        for (from, to) in res.data.chunks_exact_mut(3).zip(image.as_rgb8().unwrap().chunks_exact(3)) {
            from[0] = to[0];
            from[1] = to[1];
            from[2] = to[2];
        }

        return res;
    }
}