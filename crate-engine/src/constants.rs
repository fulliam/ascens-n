/// Number of entities per chunk (cache-line friendly: 16K)
pub const CHUNK_CAPACITY: usize = 16_384;

/// Maximum number of component types supported
pub const MAX_COMPONENTS: usize = 256;
