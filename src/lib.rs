#![cfg_attr(not(test), no_std)]
#[cfg(any(feature = "sha2", not(target_os = "solana")))]
use sha2::{Digest, Sha256};
use solana_hash::Hash;

// Only include shani_hasher_optimized if we are on x86_64 (and not on chain)
#[cfg(all(target_arch = "x86_64", not(target_os = "solana")))]
mod shani_hasher_optimized;

#[cfg(any(feature = "sha2", not(target_os = "solana")))]
#[derive(Clone, Default)]
pub struct Hasher {
    hasher: Sha256,
}

#[cfg(any(feature = "sha2", not(target_os = "solana")))]
impl Hasher {
    pub fn hash(&mut self, val: &[u8]) {
        self.hasher.update(val);
    }
    pub fn hashv(&mut self, vals: &[&[u8]]) {
        for val in vals {
            self.hash(val);
        }
    }
    pub fn result(self) -> Hash {
        let bytes: [u8; solana_hash::HASH_BYTES] = self.hasher.finalize().into();
        bytes.into()
    }
}

#[cfg(target_os = "solana")]
pub use solana_define_syscall::definitions::sol_sha256;

#[cfg(feature = "metrics")]
pub static METRICS_HITS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

#[cfg(feature = "metrics")]
pub static METRICS_MISSES: [core::sync::atomic::AtomicU64; 32] = [
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
    core::sync::atomic::AtomicU64::new(0),
];

#[inline(always)]
/// Return a Sha256 hash for the given data.
pub fn hashv(vals: &[&[u8]]) -> Hash {

    #[cfg(all(target_arch = "x86_64", not(target_os = "solana")))]
    {
        if vals.len() == 1 && vals[0].len() == 32 {

            #[cfg(feature = "metrics")]
            {
                let old_hits = METRICS_HITS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

                if old_hits % 10_000 == 0 {
                    
                    log::info!("hashv stats hits {old_hits}");
                    log::info!("hashv stats misses {:?}", METRICS_MISSES);
                }
            }

            // Since we know the array contains a single block of 32 bytes (very common in the
            // solana validator), we can use the optimized version of the hasher
            let block = unsafe { &*(vals[0].as_ptr() as *const [u8; 32]) };
            return shani_hasher_optimized::single_hash_32(block).into();
        }

        #[cfg(feature = "metrics")] {
            let len_sum = vals.iter().map(|v| v.len()).sum::<usize>().max(1).min(32);

            METRICS_MISSES[len_sum - 1].fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        }
    }
    // Perform the calculation inline, calling this from within a program is
    // not supported
    #[cfg(not(target_os = "solana"))]
    {
        let mut hasher = Hasher::default();
        hasher.hashv(vals);
        hasher.result()
    }
    // Call via a system call to perform the calculation
    #[cfg(target_os = "solana")]
    {
        let mut hash_result = [0; solana_hash::HASH_BYTES];
        unsafe {
            sol_sha256(
                vals as *const _ as *const u8,
                vals.len() as u64,
                &mut hash_result as *mut _ as *mut u8,
            );
        }
        Hash::new_from_array(hash_result)
    }
}

#[inline(always)]
/// Return a Sha256 hash for the given data.
pub fn hash(val: &[u8]) -> Hash {

    hashv(&[val])
}

/// Return the hash of the given hash extended with the given value.
pub fn extend_and_hash(id: &Hash, val: &[u8]) -> Hash {
    let mut hash_data = id.as_ref().to_vec();
    hash_data.extend_from_slice(val);
    hash(&hash_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_hash_32() {

        let inputs  = vec![
            Hash::new_unique().to_bytes(),
        ];

        const ITERS: usize = 200_000;

        // Run the sha2 implementation
        let before = std::time::Instant::now();
        let mut sw_final = Hash::new_unique();
        for _ in 0..ITERS {
            let mut hasher = Hasher::default();
            hasher.hashv(&[&inputs[0]]);
            sw_final = hasher.result()
        }
        let sw_duration = before.elapsed();

        // Run our implementation
        let before = std::time::Instant::now();
        let mut my_final = Hash::new_unique();
        for _ in 0..ITERS {
            my_final = hashv(&[&inputs[0]]);
        }
        let my_duration = before.elapsed();

        assert_eq!(my_final, sw_final, "Mismatch in hash results");

        println!("Bench {ITERS} Hash: sha2 {:?} vs our {:?}", sw_duration, my_duration);
    }

}