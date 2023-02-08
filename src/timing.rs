use std::time::{Duration, Instant};

pub fn time_one<F: FnMut()>(mut f: F) -> Duration {
    let now = Instant::now();

    f();

    return now.elapsed();
}

pub fn time_many<F: FnMut()>(mut f: F, its: usize) -> Duration {
    let now = Instant::now();

    for _ in 0..its {
        f();
    }

    return now.elapsed() / its as u32;
}