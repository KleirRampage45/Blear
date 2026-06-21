//! Simple pseudo-random number generator for click variation.

pub struct SmallRng(u64);

impl SmallRng {
    pub fn new() -> Self {
        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xdeadbeef);
        Self(seed)
    }

    pub fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let x = (self.0 >> 11) as u64;
        (x as f64) / (u64::MAX as f64)
    }

    pub fn next_i32_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max { return min; }
        min + (self.next_f64() * (max - min + 1) as f64) as i32
    }
}
