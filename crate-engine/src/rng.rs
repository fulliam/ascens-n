//! A small, dependency-free deterministic PRNG owned by the ECS [`crate::World`].
//!
//! WASM_ECS_MIGRATION_PLAN.md's Phase 4 flagged RNG determinism as a decision
//! to make *before* Phase 3 step 5 (damage resolution — crit rolls, on-hit
//! proc chances, etc.) touches any real formula, rather than defaulting to
//! whatever's convenient once that step starts. The chosen answer: Rust owns
//! a seeded, deterministic generator as a real field on `World` (not routed
//! through the generic byte-schema `Resources` system, which stores data —
//! this needs actual algorithm code, closer in kind to `entity_manager` than
//! to a `Time`/`Camera`-style resource). This buys replay/rollback and a
//! future server-authoritative netcode path without re-touching every RNG
//! call site later, at the cost of every ported system needing to pull
//! randomness from here instead of a language-native `Math.random()`/`rand`.
//!
//! SplitMix64 — not cryptographic, but fast, dependency-free, and passes
//! standard statistical test suites well enough for gameplay RNG (crit
//! rolls, proc chances). Good enough that adding the `rand` crate (and its
//! transitive dependency tree) for this alone isn't worth it.

#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// `seed=0` is NOT a degenerate all-zero state here — SplitMix64's
    /// output for state=0 is still well-distributed, but this XORs in a
    /// fixed odd constant anyway so two callers who both naively pass 0
    /// don't get a suspiciously "special" starting point.
    pub fn new(seed: u64) -> Self {
        Self { state: seed ^ 0x9E3779B97F4A7C15 }
    }

    pub fn reseed(&mut self, seed: u64) {
        self.state = seed ^ 0x9E3779B97F4A7C15;
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform float in `[0.0, 1.0)` — the `Math.random()`-shaped primitive
    /// every ported RNG call site (crit chance, proc chance, etc.) needs.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32 + 1.0)
    }

    /// `true` with probability `chance` (clamped to `[0,1]`) — the exact
    /// shape of index.html's `Math.random() < someChance` call sites.
    pub fn next_bool(&mut self, chance: f32) -> bool {
        self.next_f32() < chance.clamp(0.0, 1.0)
    }
}

impl Default for Rng {
    fn default() -> Self {
        // A fixed, non-time-based default seed — a fresh World::new() is
        // reproducible run-to-run even if nobody calls reseed() explicitly.
        // Callers who want real per-session randomness (the common case for
        // a single-player game today) reseed from a JS-supplied value (e.g.
        // Date.now() or crypto.getRandomValues()) right after construction.
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..200 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        let seq_a: Vec<u64> = (0..20).map(|_| a.next_u64()).collect();
        let seq_b: Vec<u64> = (0..20).map(|_| b.next_u64()).collect();
        assert_ne!(seq_a, seq_b);
    }

    #[test]
    fn reseed_restarts_the_sequence() {
        let mut a = Rng::new(7);
        let first_run: Vec<u64> = (0..10).map(|_| a.next_u64()).collect();
        a.reseed(7);
        let second_run: Vec<u64> = (0..10).map(|_| a.next_u64()).collect();
        assert_eq!(first_run, second_run);
    }

    #[test]
    fn next_f32_stays_in_unit_range() {
        let mut r = Rng::new(7);
        for _ in 0..10_000 {
            let v = r.next_f32();
            assert!(v >= 0.0 && v < 1.0, "next_f32() out of range: {v}");
        }
    }

    #[test]
    fn next_bool_respects_extreme_chances() {
        let mut r = Rng::new(99);
        for _ in 0..500 {
            assert!(!r.next_bool(0.0));
        }
        for _ in 0..500 {
            assert!(r.next_bool(1.0));
        }
    }

    #[test]
    fn next_bool_roughly_matches_probability() {
        // Not a strict statistical test (would be flaky) — just a sanity
        // check that a 50% chance isn't wildly skewed over a large sample.
        let mut r = Rng::new(1234);
        let hits = (0..100_000).filter(|_| r.next_bool(0.5)).count();
        let frac = hits as f64 / 100_000.0;
        assert!((0.48..0.52).contains(&frac), "50% chance sampled as {frac}");
    }
}
