use std::time::{Instant, Duration};

pub struct FPSCounter {
    start: Instant,
    last_frame: Instant,

    pub frames: usize,
    pub target_fps: f64,
    target_duration: Duration,
    pub current_fps: f64
}

impl FPSCounter {
    pub fn new(target_fps: f64) -> FPSCounter {
        return FPSCounter { 
            start: Instant::now(), 
            last_frame: Instant::now(), 
            frames: 0,
            target_fps, 
            target_duration: Duration::from_secs_f64(1.0 / target_fps),
            current_fps: target_fps
        }
    }

    pub fn start(&mut self) {
        self.start = Instant::now();
        self.last_frame = Instant::now();
        self.frames = 1;
    }

    /*
    Checks the elapsed time and returns true if a new frame is needed in order to mantain target FPS
    */
    pub fn update(&mut self) -> bool {
        let elapsed = self.last_frame.elapsed();

        if self.target_duration <= elapsed {
            self.last_frame = Instant::now();
            self.frames += 1;

            let new_fps = 1.0 / elapsed.as_secs_f64();
            let delta = new_fps - self.current_fps;
            self.current_fps += delta / 10.0; // Weight mean

            return true;
        }

        return false;
    }
}