//! ISA-dispatched unpack functions for low-precision data representations.
//!
//! Every unpack operation has the same shape — probe the ISA, call the matching
//! kernel inside its `#[target_feature]` scope, else run the scalar fallback —
//! so the six public entry points are generated from one parameterised macro
//! instead of six near-identical bodies. The operation's dimensions (source and
//! destination element types, AVX-512 feature guard, scalar fallback) are macro
//! parameters; the public names are unchanged.

use crate::types::{Bf16, Bf4, Bf8, F32, F4, F8};

#[cfg(target_arch = "x86_64")]
use super::arch::{has_avx2, has_avx512bw, has_avx512f};

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use super::unsafe_intrinsics;

#[cfg(not(target_arch = "aarch64"))]
use crate::convert::{widen_finite, widen_finite_high_word, widen_high_word};

/// Generate one public unpack entry point.
///
/// `$avx512_guard` selects the AVX-512 feature the kernel needs (`avx512bw` for
/// the byte-shuffle kernels, `avx512f` for the float converts). `$scalar` is the
/// fallback body, written against the two slice bindings `$packed`/`$unpacked`
/// the macro supplies.
macro_rules! dispatch_unpack {
    (
        $(#[$meta:meta])*
        $name:ident, $src:ty, $dst:ty, $avx512_guard:path,
        |$packed:ident, $unpacked:ident| $scalar:block
    ) => {
        $(#[$meta])*
        #[inline]
        pub fn $name(packed: &[$src], unpacked: &mut [$dst]) {
            #[cfg(target_arch = "x86_64")]
            {
                if $avx512_guard() {
                    // SAFETY: the guard above confirms the required AVX-512 ISA
                    // is available; the kernel bounds reads/writes to the slice
                    // lengths.
                    unsafe {
                        unsafe_intrinsics::avx512::$name(packed, unpacked);
                    }
                    return;
                }
                if has_avx2() {
                    // SAFETY: `has_avx2()` confirmed the ISA; the kernel bounds
                    // reads/writes to the slice lengths.
                    unsafe {
                        unsafe_intrinsics::avx2::$name(packed, unpacked);
                    }
                    return;
                }
            }
            #[cfg(target_arch = "aarch64")]
            {
                // SAFETY: NEON is baseline on aarch64; the kernel bounds
                // reads/writes to the slice lengths.
                unsafe {
                    unsafe_intrinsics::neon::$name(packed, unpacked);
                }
                return;
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                let $packed = packed;
                let $unpacked = unpacked;
                $scalar
            }
        }
    };
}

dispatch_unpack!(
    /// Unpacks Bf8 elements to Bf16.
    unpack_bf8_to_bf16, Bf8, Bf16, has_avx512bw,
    |packed, unpacked| {
        let n = packed.len().min(unpacked.len());
        for i in 0..n {
            unpacked[i] = Bf16(widen_high_word::<5, 2>(u32::from(packed[i].0)));
        }
    }
);

dispatch_unpack!(
    /// Unpacks Bf4 elements to Bf16.
    unpack_bf4_to_bf16, Bf4, Bf16, has_avx512bw,
    |packed, unpacked| {
        let n = packed.len().min(unpacked.len());
        for i in 0..n {
            unpacked[i] = Bf16(widen_finite_high_word::<2, 1>(u32::from(packed[i].0)));
        }
    }
);

dispatch_unpack!(
    /// Unpacks packed 4-bit Bf4 pairs (stored 2 per byte) into a Bf16 slice.
    unpack_bf4_to_bf16_packed, u8, Bf16, has_avx512bw,
    |packed, unpacked| {
        let n = packed.len().min(unpacked.len() / 2);
        for i in 0..n {
            let byte = packed[i];
            unpacked[2 * i] = Bf16(widen_finite_high_word::<2, 1>(u32::from(byte & 0x0f)));
            unpacked[2 * i + 1] = Bf16(widen_finite_high_word::<2, 1>(u32::from((byte >> 4) & 0x0f)));
        }
    }
);

dispatch_unpack!(
    /// Unpacks F4 elements to F32.
    unpack_f4_to_f32, F4, F32, has_avx512f,
    |packed, unpacked| {
        let n = packed.len().min(unpacked.len());
        for i in 0..n {
            unpacked[i] = F32(packed[i].to_f32());
        }
    }
);

dispatch_unpack!(
    /// Unpacks packed 4-bit F4 pairs (stored 2 per byte) into an F32 slice.
    unpack_f4_to_f32_packed, u8, F32, has_avx512f,
    |packed, unpacked| {
        let n = packed.len().min(unpacked.len() / 2);
        for i in 0..n {
            let byte = packed[i];
            unpacked[2 * i] = F32(F4(byte & 0x0f).to_f32());
            unpacked[2 * i + 1] = F32(F4((byte >> 4) & 0x0f).to_f32());
        }
    }
);

dispatch_unpack!(
    /// Unpacks F8 elements to F32.
    unpack_f8_to_f32, F8, F32, has_avx512f,
    |packed, unpacked| {
        static TABLE_BITS: [u32; 256] = {
            let mut t = [0u32; 256];
            let mut idx = 0;
            while idx < 256 {
                t[idx] = widen_finite::<4, 3>(idx as u32);
                idx += 1;
            }
            t
        };
        let n = packed.len().min(unpacked.len());
        for i in 0..n {
            unpacked[i] = F32(f32::from_bits(TABLE_BITS[packed[i].0 as usize]));
        }
    }
);
