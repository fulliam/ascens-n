/// Growable bitset for component masks — O(n/64) operations
#[derive(Clone, Default, Debug)]
pub struct ComponentMask {
    words: Vec<u64>,
}

impl ComponentMask {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn ensure(&mut self, bit: usize) {
        let index = bit / 64;
        if self.words.len() <= index {
            self.words.resize(index + 1, 0);
        }
    }

    #[inline]
    pub fn set(&mut self, bit: usize) {
        self.ensure(bit);
        self.words[bit / 64] |= 1u64 << (bit % 64);
    }

    #[inline]
    pub fn clear(&mut self, bit: usize) {
        if bit / 64 < self.words.len() {
            self.words[bit / 64] &= !(1u64 << (bit % 64));
        }
    }

    #[inline]
    pub fn contains(&self, bit: usize) -> bool {
        self.words
            .get(bit / 64)
            .map(|w| (w & (1u64 << (bit % 64))) != 0)
            .unwrap_or(false)
    }

    /// Returns true if self has all bits set in `query`
    #[inline]
    pub fn matches(&self, query: &ComponentMask) -> bool {
        for (i, q) in query.words.iter().enumerate() {
            let current = self.words.get(i).copied().unwrap_or(0);
            if (current & q) != *q {
                return false;
            }
        }
        true
    }

    /// Returns true if self and other share NO bits
    pub fn is_disjoint(&self, other: &ComponentMask) -> bool {
        let min_len = self.words.len().min(other.words.len());
        for i in 0..min_len {
            if (self.words[i] & other.words[i]) != 0 {
                return false;
            }
        }
        true
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|w| *w == 0)
    }

    pub fn count_ones(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }
}

impl PartialEq for ComponentMask {
    fn eq(&self, other: &Self) -> bool {
        let max_len = self.words.len().max(other.words.len());
        for i in 0..max_len {
            let a = self.words.get(i).copied().unwrap_or(0);
            let b = other.words.get(i).copied().unwrap_or(0);
            if a != b { return false; }
        }
        true
    }
}

impl Eq for ComponentMask {}
