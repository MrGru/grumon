//! Tiny deterministic RNG stored inside the battle state (game-systems §7.5).

use serde::{Deserialize, Serialize};

/// SplitMix64: fast, good enough for gameplay, trivially serialisable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Seed derived from a string ID and a counter (FNV-1a).
    pub fn seed_from(id: &str, counter: u32) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in id.bytes().chain(counter.to_le_bytes()) {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hash
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform integer in `lo..=hi`.
    pub fn range(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() % (hi - lo + 1) as u64) as u32
    }

    /// True with `percent` % probability.
    pub fn chance(&mut self, percent: u32) -> bool {
        self.range(1, 100) <= percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_in_range() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            let x = a.range(95, 105);
            assert_eq!(x, b.range(95, 105));
            assert!((95..=105).contains(&x));
        }
        assert_ne!(Rng::seed_from("a", 0), Rng::seed_from("b", 0));
    }
}
