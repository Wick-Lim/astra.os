// random.rs - Random number generation for ASTRA.OS
// For now, use a simple pseudo-random number generator

static mut SEED: u64 = 0x123456789abcdef0;

pub fn fill_bytes(bytes: &mut [u8]) {
    // Simple LCG (Linear Congruential Generator)
    unsafe {
        for byte in bytes.iter_mut() {
            SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *byte = (SEED >> 56) as u8;
        }
    }
}
