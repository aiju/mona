use std::ops::{Add, Mul};

pub struct Sampler<T> {
    pub mode: SamplerMode,
    pub keyframes: Vec<f64>,
    pub samples: Vec<T>,
    pub time: f64,
    pub index: usize,
}

pub enum SamplerMode {
    Step,
    Linear,
}

impl<T: Copy + Add<Output = T> + Mul<f64, Output = T>> Sampler<T> {
    pub fn sample(&self) -> T {
        if self.index >= self.keyframes.len() - 1 {
            self.samples[self.keyframes.len() - 1]
        } else if self.index == 0 && self.time < self.keyframes[0] {
            self.samples[0]
        } else {
            let t1 = self.keyframes[self.index];
            let t2 = self.keyframes[self.index + 1];
            let t = (self.time - t1) / (t2 - t1);
            debug_assert!(t >= 0.0 && t <= 1.0);
            let s1 = self.samples[self.index];
            let s2 = self.samples[self.index + 1];
            match self.mode {
                SamplerMode::Step => s1,
                SamplerMode::Linear => s1 * (1.0 - t) + s2 * t,
            }
        }
    }
    pub fn advance(&mut self, delta: f64) {
        debug_assert!(delta >= 0.0);
        self.time += delta;
        while self.index < self.keyframes.len() - 1 && self.time >= self.keyframes[self.index + 1] {
            self.index += 1;
        }
    }
}
