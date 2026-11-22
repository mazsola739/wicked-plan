//! Small seeded random number generator (mulberry32).
//! Rules never use any other randomness: the same seed gives the same game,
//! so tests, replays and the players of one race all see the same puzzles.

#[derive(Clone, Debug)]
pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Fold the high bits in so that large seeds still differ.
        Self { state: (seed ^ (seed >> 32)) as u32 }
    }

    /// Float in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6d2b_79f5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }

    /// Integer in [min, max], both inclusive.
    pub fn int(&mut self, min: i64, max: i64) -> i64 {
        assert!(max >= min, "int(): max {max} < min {min}");
        min + (self.next_f64() * (max - min + 1) as f64).floor() as i64
    }

    pub fn index(&mut self, len: usize) -> usize {
        assert!(len > 0, "index(): empty range");
        self.int(0, len as i64 - 1) as usize
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.index(items.len())]
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.index(i + 1);
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_numbers() {
        let a: Vec<f64> = (0..5)
            .map({
                let mut r = Rng::new(42);
                move |_| r.next_f64()
            })
            .collect();
        let mut r = Rng::new(42);
        let b: Vec<f64> = (0..5).map(|_| r.next_f64()).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn int_stays_in_range_and_hits_both_ends() {
        let mut r = Rng::new(7);
        let mut seen = [false; 6];
        for _ in 0..1000 {
            let v = r.int(1, 6);
            assert!((1..=6).contains(&v));
            seen[(v - 1) as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn shuffle_keeps_items() {
        let mut r = Rng::new(3);
        let mut v: Vec<i32> = (0..20).collect();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, (0..20).collect::<Vec<_>>());
    }
}
