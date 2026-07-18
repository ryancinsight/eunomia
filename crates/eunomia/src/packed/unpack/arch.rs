//! Runtime ISA feature detection for x86/x86_64.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub(super) fn has_avx512bw() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(feature = "std")]
        {
            std::is_x86_feature_detected!("avx512bw") && std::is_x86_feature_detected!("avx512vl")
        }
        #[cfg(not(feature = "std"))]
        {
            // Match the `std` runtime check and the intrinsics' `#[target_feature]`
            // requirement: `avx512vl` is needed for the 128/256-bit-width ops.
            cfg!(target_feature = "avx512bw") && cfg!(target_feature = "avx512vl")
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub(super) fn has_avx512f() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(feature = "std")]
        {
            std::is_x86_feature_detected!("avx512f") && std::is_x86_feature_detected!("avx512vl")
        }
        #[cfg(not(feature = "std"))]
        {
            // `avx512vl` mirrors the `std` check and the intrinsics' requirement.
            cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512vl")
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub(super) fn has_avx2() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(feature = "std")]
        {
            std::is_x86_feature_detected!("avx2")
        }
        #[cfg(not(feature = "std"))]
        {
            cfg!(target_feature = "avx2")
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}
