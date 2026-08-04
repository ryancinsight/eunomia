//! Use Eunomia's complex vocabulary in a solver-shaped calculation.

use eunomia::{Complex32, Complex64};

fn main() {
    let phasor = Complex64::new(3.0, 4.0);
    let response = phasor * Complex64::new(0.0, 1.0);
    let recovered = response / Complex64::new(0.0, 1.0);

    eunomia::assert_relative_eq!(recovered.re, phasor.re, epsilon = 1.0e-12);
    eunomia::assert_relative_eq!(recovered.im, phasor.im, epsilon = 1.0e-12);
    println!(
        "phasor = {phasor}, response = {response}, norm = {}",
        phasor.norm()
    );

    // Complex32 is a #[repr(C)] Pod pair, so bytemuck can marshal it without
    // an intermediate third-party complex representation.
    let samples = [Complex32::new(1.0, -2.0), Complex32::new(0.5, 3.0)];
    let bytes = bytemuck::cast_slice::<Complex32, u8>(&samples);
    let restored = bytemuck::cast_slice::<u8, Complex32>(bytes);
    assert_eq!(restored, samples);
    println!(
        "marshalled {} Complex32 values as {} bytes",
        restored.len(),
        bytes.len()
    );
}
