// Inspect exact and rounded conversions through Eunomia's native kernel.

use eunomia::convert::{narrow, widen};
use eunomia::{Bf16, F16};

fn main() {
    let one_f16 = narrow::<5, 10>(1.0_f32.to_bits());
    assert_eq!(one_f16, 0x3C00);
    assert_eq!(f32::from_bits(widen::<5, 10>(one_f16)), 1.0);

    let values = [0.1_f32, 0.5, 1.0, 3.25, 10.0];
    for value in values {
        let f16 = F16::from_f32(value);
        let bf16 = Bf16::from_f32(value);
        println!(
            "{value:>4.2} -> F16 0x{:04X} ({:.7}), Bf16 0x{:04X} ({:.7})",
            f16.to_bits(),
            f16.to_f32(),
            bf16.to_bits(),
            bf16.to_f32()
        );
    }

    // Exact powers of two survive both reduced formats unchanged.
    assert_eq!(F16::from_f32(0.5).to_f32(), 0.5);
    assert_eq!(Bf16::from_f32(0.5).to_f32(), 0.5);

    // 1.0 is exactly halfway between binary16's 1.0 and the next grid point;
    // ties-to-even keeps the even significand, so the midpoint rounds down.
    let midpoint = 1.0_f32 + 2.0_f32.powi(-11);
    assert_eq!(F16::from_f32(midpoint).to_bits(), 0x3C00);

    // The error is bounded by half an ulp of the destination grid. Around 0.1,
    // binary16 has exponent -4, so one ulp is 2^(-4 - 10).
    let value = 0.1_f32;
    let rounded = F16::from_f32(value).to_f32();
    let half_ulp = 2.0_f32.powi(-15);
    assert!((rounded - value).abs() <= half_ulp);
}
