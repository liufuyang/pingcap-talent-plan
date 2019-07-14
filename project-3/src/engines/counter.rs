use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LengthCount {
    /// Total length of a log
    len: usize,
    /// Length of garbage
    len_garbage: usize,
}

impl LengthCount {
    pub fn new () -> Self {
        LengthCount{ len: 0, len_garbage: 0}
    }

    pub fn effective_len(&self) -> usize {
        self.len - self.len_garbage
    }

    pub fn increase_len(&mut self) {
        self.len += 1;
    }

    pub fn increase_garbage_len(&mut self) {
        self.len_garbage += 1;
    }

    pub fn increase_len_with_garbage(&mut self) {
        self.len += 1;
        self.len_garbage += 1;
    }

    pub fn garbage_rate(&self) -> f64{
        self.len_garbage as f64 / self.len as f64
    }
}