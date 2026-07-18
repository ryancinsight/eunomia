use crate::convert::{widen_finite, widen_finite_high_word, widen_high_word};
use crate::types::{Bf16, Bf4, Bf8, F32, F4, F8};

/// # Safety
/// Callers must ensure the running CPU supports AVX2 (the dispatcher checks
/// `has_avx2()`). Reads and writes stay within the shorter of the two slices.
#[target_feature(enable = "avx2")]
pub unsafe fn unpack_bf8_to_bf16(packed: &[Bf8], unpacked: &mut [Bf16]) {
    use core::arch::x86_64::*;

    let len = packed.len().min(unpacked.len());
    let mut i = 0;

    static TABLE: [i32; 256] = {
        let mut table = [0i32; 256];
        let mut index = 0;
        while index < table.len() {
            table[index] = widen_high_word::<5, 2>(index as u32) as i32;
            index += 1;
        }
        table
    };

    while i + 16 <= len {
        let input = _mm_loadu_si128(packed.as_ptr().add(i) as *const __m128i);
        let lower_indices = _mm256_cvtepu8_epi32(input);
        let upper_indices = _mm256_cvtepu8_epi32(_mm_srli_si128(input, 8));
        let lower = _mm256_i32gather_epi32(TABLE.as_ptr(), lower_indices, 4);
        let upper = _mm256_i32gather_epi32(TABLE.as_ptr(), upper_indices, 4);
        let packed_words = _mm256_packus_epi32(lower, upper);
        let ordered = _mm256_permute4x64_epi64(packed_words, 0xD8);
        _mm256_storeu_si256(unpacked.as_mut_ptr().add(i) as *mut __m256i, ordered);
        i += 16;
    }
    for j in i..len {
        unpacked[j] = Bf16(widen_high_word::<5, 2>(u32::from(packed[j].0)));
    }
}

/// # Safety
/// Callers must ensure the running CPU supports AVX2 (the dispatcher checks
/// `has_avx2()`). Reads and writes stay within the shorter of the two slices.
#[target_feature(enable = "avx2")]
pub unsafe fn unpack_bf4_to_bf16(packed: &[Bf4], unpacked: &mut [Bf16]) {
    use core::arch::x86_64::*;
    let len = packed.len().min(unpacked.len());
    let mut i = 0;

    static TABLE_LO: [u8; 16] = {
        let mut t = [0u8; 16];
        let mut idx = 0;
        while idx < 16 {
            t[idx] = (widen_finite_high_word::<2, 1>(idx as u32) & 0xFF) as u8;
            idx += 1;
        }
        t
    };
    static TABLE_HI: [u8; 16] = {
        let mut t = [0u8; 16];
        let mut idx = 0;
        while idx < 16 {
            t[idx] = (widen_finite_high_word::<2, 1>(idx as u32) >> 8) as u8;
            idx += 1;
        }
        t
    };

    let table_lo = _mm_loadu_si128(TABLE_LO.as_ptr() as *const _);
    let table_hi = _mm_loadu_si128(TABLE_HI.as_ptr() as *const _);
    let mask_0f = _mm_set1_epi8(0x0F);

    while i + 16 <= len {
        let ptr = packed.as_ptr().add(i) as *const __m128i;
        let v_in = _mm_loadu_si128(ptr);
        let indices = _mm_and_si128(v_in, mask_0f);

        let res_lo = _mm_shuffle_epi8(table_lo, indices);
        let res_hi = _mm_shuffle_epi8(table_hi, indices);

        let out_lo = _mm_unpacklo_epi8(res_lo, res_hi);
        let out_hi = _mm_unpackhi_epi8(res_lo, res_hi);

        _mm_storeu_si128(unpacked.as_mut_ptr().add(i) as *mut _, out_lo);
        _mm_storeu_si128(unpacked.as_mut_ptr().add(i + 8) as *mut _, out_hi);

        i += 16;
    }
    for j in i..len {
        let b = packed[j].0;
        unpacked[j] = Bf16(widen_finite_high_word::<2, 1>(b as u32));
    }
}

/// # Safety
/// Callers must ensure the running CPU supports AVX2 (the dispatcher checks
/// `has_avx2()`). Reads and writes stay within the shorter of the two slices.
#[target_feature(enable = "avx2")]
pub unsafe fn unpack_bf4_to_bf16_packed(packed: &[u8], unpacked: &mut [Bf16]) {
    use core::arch::x86_64::*;
    let len = packed.len();
    let n = len.min(unpacked.len() / 2);
    let mut i = 0;

    static TABLE_LO: [u8; 16] = {
        let mut t = [0u8; 16];
        let mut idx = 0;
        while idx < 16 {
            t[idx] = (widen_finite_high_word::<2, 1>(idx as u32) & 0xFF) as u8;
            idx += 1;
        }
        t
    };
    static TABLE_HI: [u8; 16] = {
        let mut t = [0u8; 16];
        let mut idx = 0;
        while idx < 16 {
            t[idx] = (widen_finite_high_word::<2, 1>(idx as u32) >> 8) as u8;
            idx += 1;
        }
        t
    };

    let table_lo = _mm_loadu_si128(TABLE_LO.as_ptr() as *const _);
    let table_hi = _mm_loadu_si128(TABLE_HI.as_ptr() as *const _);
    let mask_0f = _mm_set1_epi8(0x0F);

    while i + 16 <= n {
        let ptr = packed.as_ptr().add(i) as *const __m128i;
        let v = _mm_loadu_si128(ptr);

        let low_nibbles = _mm_and_si128(v, mask_0f);
        let high_nibbles = _mm_and_si128(_mm_srli_epi16(v, 4), mask_0f);

        let res_lo = _mm_unpacklo_epi8(low_nibbles, high_nibbles);
        let res_hi = _mm_unpackhi_epi8(low_nibbles, high_nibbles);

        let res_lo_lo = _mm_shuffle_epi8(table_lo, res_lo);
        let res_lo_hi = _mm_shuffle_epi8(table_hi, res_lo);
        let out_lo0 = _mm_unpacklo_epi8(res_lo_lo, res_lo_hi);
        let out_lo1 = _mm_unpackhi_epi8(res_lo_lo, res_lo_hi);

        let res_hi_lo = _mm_shuffle_epi8(table_lo, res_hi);
        let res_hi_hi = _mm_shuffle_epi8(table_hi, res_hi);
        let out_hi0 = _mm_unpacklo_epi8(res_hi_lo, res_hi_hi);
        let out_hi1 = _mm_unpackhi_epi8(res_hi_lo, res_hi_hi);

        _mm_storeu_si128(unpacked.as_mut_ptr().add(2 * i) as *mut _, out_lo0);
        _mm_storeu_si128(unpacked.as_mut_ptr().add(2 * i + 8) as *mut _, out_lo1);
        _mm_storeu_si128(unpacked.as_mut_ptr().add(2 * i + 16) as *mut _, out_hi0);
        _mm_storeu_si128(unpacked.as_mut_ptr().add(2 * i + 24) as *mut _, out_hi1);

        i += 16;
    }

    for j in i..n {
        let byte = packed[j];
        unpacked[2 * j] = Bf16(widen_finite_high_word::<2, 1>((byte & 0x0F) as u32));
        unpacked[2 * j + 1] = Bf16(widen_finite_high_word::<2, 1>(((byte >> 4) & 0x0F) as u32));
    }
}

/// # Safety
/// Callers must ensure the running CPU supports AVX2 (the dispatcher checks
/// `has_avx2()`). Reads and writes stay within the shorter of the two slices.
#[target_feature(enable = "avx2")]
pub unsafe fn unpack_f4_to_f32(packed: &[F4], unpacked: &mut [F32]) {
    let len = packed.len().min(unpacked.len());
    let mut i = 0;

    static TABLE_BITS: [u32; 16] = {
        let mut t = [0u32; 16];
        let mut idx = 0;
        while idx < 16 {
            t[idx] = widen_finite::<3, 0>(idx as u32);
            idx += 1;
        }
        t
    };

    while i + 8 <= len {
        let ptr = packed.as_ptr().add(i) as *const core::arch::x86_64::__m128i;
        let v_in = core::arch::x86_64::_mm_loadl_epi64(ptr);
        let v_u32 = core::arch::x86_64::_mm256_cvtepu8_epi32(v_in);
        let indices = core::arch::x86_64::_mm256_and_si256(
            v_u32,
            core::arch::x86_64::_mm256_set1_epi32(0x0f),
        );

        let result =
            core::arch::x86_64::_mm256_i32gather_ps(TABLE_BITS.as_ptr() as *const _, indices, 4);

        let out_ptr = unpacked.as_mut_ptr().add(i) as *mut f32;
        core::arch::x86_64::_mm256_storeu_ps(out_ptr, result);
        i += 8;
    }

    for j in i..len {
        unpacked[j] = F32(f32::from_bits(TABLE_BITS[(packed[j].0 & 0x0f) as usize]));
    }
}

/// # Safety
/// Callers must ensure the running CPU supports AVX2 (the dispatcher checks
/// `has_avx2()`). Reads and writes stay within the shorter of the two slices.
#[target_feature(enable = "avx2")]
pub unsafe fn unpack_f4_to_f32_packed(packed: &[u8], unpacked: &mut [F32]) {
    let len = packed.len();
    let n = len.min(unpacked.len() / 2);
    let mut i = 0;

    static TABLE_BITS: [u32; 16] = {
        let mut t = [0u32; 16];
        let mut idx = 0;
        while idx < 16 {
            t[idx] = widen_finite::<3, 0>(idx as u32);
            idx += 1;
        }
        t
    };

    while i + 8 <= n {
        let bytes_ptr = packed.as_ptr().add(i);
        let v_bytes = core::arch::x86_64::_mm_loadl_epi64(bytes_ptr as *const _);
        let v_u32 = core::arch::x86_64::_mm256_cvtepu8_epi32(v_bytes);

        let low_nibbles = core::arch::x86_64::_mm256_and_si256(
            v_u32,
            core::arch::x86_64::_mm256_set1_epi32(0x0f),
        );
        let high_nibbles = core::arch::x86_64::_mm256_and_si256(
            core::arch::x86_64::_mm256_srli_epi32(v_u32, 4),
            core::arch::x86_64::_mm256_set1_epi32(0x0f),
        );

        let val_low = core::arch::x86_64::_mm256_i32gather_ps(
            TABLE_BITS.as_ptr() as *const _,
            low_nibbles,
            4,
        );
        let val_high = core::arch::x86_64::_mm256_i32gather_ps(
            TABLE_BITS.as_ptr() as *const _,
            high_nibbles,
            4,
        );

        let val_lo_u = core::arch::x86_64::_mm256_castps_si256(val_low);
        let val_hi_u = core::arch::x86_64::_mm256_castps_si256(val_high);

        let res0 = core::arch::x86_64::_mm256_unpacklo_epi32(val_lo_u, val_hi_u);
        let res1 = core::arch::x86_64::_mm256_unpackhi_epi32(val_lo_u, val_hi_u);

        let out0 = core::arch::x86_64::_mm256_permute2x128_si256(res0, res1, 0x20);
        let out1 = core::arch::x86_64::_mm256_permute2x128_si256(res0, res1, 0x31);

        let out_ptr0 = unpacked.as_mut_ptr().add(2 * i) as *mut core::arch::x86_64::__m256i;
        core::arch::x86_64::_mm256_storeu_si256(out_ptr0, out0);
        let out_ptr1 = unpacked.as_mut_ptr().add(2 * i + 8) as *mut core::arch::x86_64::__m256i;
        core::arch::x86_64::_mm256_storeu_si256(out_ptr1, out1);

        i += 8;
    }

    for j in i..n {
        let byte = packed[j];
        unpacked[2 * j] = F32(f32::from_bits(TABLE_BITS[(byte & 0x0F) as usize]));
        unpacked[2 * j + 1] = F32(f32::from_bits(TABLE_BITS[((byte >> 4) & 0x0F) as usize]));
    }
}

/// # Safety
/// Callers must ensure the running CPU supports AVX2 (the dispatcher checks
/// `has_avx2()`). Reads and writes stay within the shorter of the two slices.
#[target_feature(enable = "avx2")]
pub unsafe fn unpack_f8_to_f32(packed: &[F8], unpacked: &mut [F32]) {
    let len = packed.len().min(unpacked.len());
    let mut i = 0;

    static TABLE_BITS: [u32; 256] = {
        let mut t = [0u32; 256];
        let mut idx = 0;
        while idx < 256 {
            t[idx] = widen_finite::<4, 3>(idx as u32);
            idx += 1;
        }
        t
    };

    while i + 8 <= len {
        let ptr = packed.as_ptr().add(i) as *const core::arch::x86_64::__m128i;
        let v_in = core::arch::x86_64::_mm_loadl_epi64(ptr);
        let v_u32 = core::arch::x86_64::_mm256_cvtepu8_epi32(v_in);

        let result =
            core::arch::x86_64::_mm256_i32gather_ps(TABLE_BITS.as_ptr() as *const _, v_u32, 4);

        let out_ptr = unpacked.as_mut_ptr().add(i) as *mut f32;
        core::arch::x86_64::_mm256_storeu_ps(out_ptr, result);
        i += 8;
    }

    for j in i..len {
        unpacked[j] = F32(f32::from_bits(TABLE_BITS[packed[j].0 as usize]));
    }
}
