# ADR 0003: Native byte-layout & reduced-precision vocabulary

- Status: Accepted (implementation staged; slice 1 landed)
- Date: 2026-07-18
- Class: [arch] (grows eunomia's owned surface; shrinks external deps; ripples to
  every reduced-precision / GPU-byte consumer)
- Supersedes/relates: [ADR 0001](0001-eunomia-datatype-ssot.md) (datatype SSOT),
  [gap_audit.md](../../gap_audit.md) §Byte-layout / reduced-precision.

## Context

Eunomia is the datatype law but does not fully own two datatype concerns it is
the natural home for:

1. **Reduced precision.** `F16`/`Bf16` are transparent wrappers over
   `half::f16`/`half::bf16`; `half` is a hard runtime dependency. The sub-byte
   `F8`/`Bf8`/`F4`/`Bf4` are already native but hand-rolled four times, and they
   **truncate** instead of rounding to nearest-even and pin an unpinned,
   non-standard special-value convention.
2. **Byte layout / transmutation.** Eunomia borrows `bytemuck::{Pod,Zeroable}`
   for its scalar markers and owns no "safe to view as bytes" / slice-reinterpret
   vocabulary; the stack reaches for `bytemuck` directly at ~530 call sites.

The audit established the constraints that bound the decision:

- Eunomia targets **stable 1.95.0** → the std `core::mem::TransmuteFrom` trait
  (nightly `#![feature(transmutability)]`, experimental, no stabilization track,
  open soundness hole [#129097]) is **not adoptable**.
- **wgpu/metal/cuda contractually require `bytemuck::Pod`** at the buffer
  boundary (all ~530 stack sites are internal GPU-ABI structs, no cross-crate
  public `Pod` type) → bytemuck cannot be removed, only bridged.
- **`half` has no equivalent external lock-in**; its stack surface is a bounded
  set of conversion methods and the public `f16`/`bf16` element type in
  hermes → leto → coeus/apollo. It is fully replaceable.
- The stack's entire `zerocopy` need is one call (`IntoBytes::as_bytes`, in
  out-of-scope consus); `half` rounds round-to-nearest-ties-to-even, giving an
  exact differential oracle for a native kernel.

## Decision

**D1 — Own reduced-precision conversion natively; retire the `half` runtime
dependency.** One generic const-parameterized IEEE-754 narrow/widen kernel
(`convert::{narrow, widen}`, `<const E, const M>`, round-to-nearest-ties-to-even,
subnormals, inf/NaN, f32-subnormal handling) is the single conversion SSOT.
`binary16` (`E=5,M=10`) and `bfloat16` (`E=8,M=7`) instantiate it; the sub-byte
formats fold onto it, deleting the four hand-rolled copies and the truncation
defect. `half` demotes to a **dev-dependency oracle** + an optional
`half-interop` feature (bridge trait impls) held only across the consumer
migration window, then dropped.

**D2 — Own a byte-layout vocabulary at zerocopy's checked tier; bridge, do not
replace, bytemuck.** Eunomia gains marker traits (`Zeroable`, `Pod`-equivalent)
and the safe reinterpretation the stack actually uses (`cast_slice`(+mut),
`bytes_of`, `from_bytes`, unaligned read), machine-checked for eunomia's own
types via the existing `const _` size/align assertions + per-impl `// SAFETY:`.
A default `bytemuck-interop` feature blanket-bridges eunomia markers ↔
`bytemuck::{Pod,Zeroable}` so eunomia types keep satisfying the wgpu/metal/cuda
`bytemuck::Pod` contract. Scope is bounded to what the audit found consumers use
— **no derive macro and no OCP/checked-transmute surface is built speculatively.**

**D3 — Pin the sub-byte special-value convention explicitly; add OCP-MXFP as a
distinct format family only when a consumer needs it.** The existing
`F8`(E4M3)/`Bf8`(E5M2)/`F4`(E3M0)/`Bf4`(E2M1) keep their current *IEEE-style,
reserved-top-exponent, no-infinity* convention, now **documented and
reference-tested**. OCP-MXFP FP8/FP4 (no infinity; the emerging GPU-quantization
standard, which eunomia's `F4`=E3M0 matches no format of) is added as new types
selected through the kernel's special-value policy parameter **when coeus/
hephaestus quantization requires it** — not before.

**D4 — `TransmuteFrom` and `zerocopy` are reference tier only**, not
dependencies: `TransmuteFrom` is an internal, nightly-gated audit aid at most;
`zerocopy`'s design (derive + `KnownLayout` compile-checked safety) is the model
D2's checked tier emulates on stable.

## Options considered

- **Adopt `std::mem::TransmuteFrom` as the transmutation foundation** — rejected:
  nightly-only, unstable, unsound today; eunomia must stay stable (D4).
- **Reimplement all of bytemuck/zerocopy in eunomia and drop both** — rejected:
  over-engineering (anti-cargo-cult) and impossible for the wgpu boundary that
  fixes `bytemuck::Pod`. D2 bridges instead and scopes to used surface.
- **Keep wrapping `half`; only fix the sub-byte truncation** — rejected: leaves a
  removable runtime dep on the datatype-law crate and forgoes the consolidation
  of five conversion implementations into one kernel (D1).
- **Re-back `F16`/`Bf16` on `u16` and drop `half` in one commit** — rejected:
  breaking field-type change with ≥1 external constructor (`hermes` test builds
  `Bf16(half::bf16::…)`); sequenced as a co-evolution unit instead (E-025).
- **Match OCP-MXFP for the existing sub-byte types now** — rejected: a behavior
  change with GPU-quantization impact and no current consumer; additive new
  types when needed (D3, justified-constructs).

## Consequences

- Eunomia's external deps shrink from `{half, bytemuck, libm, rkyv?}` toward
  `{libm, rkyv?}` + optional interop features; the datatype law owns its
  conversions and byte vocabulary.
- One conversion kernel replaces five implementations and fixes the
  truncation/convention defects (G-C2/G-C3/G-A3).
- Consumers migrate `half::f16` → `eunomia::F16` along the hermes → leto →
  coeus/apollo chain (co-evolution units), each keeping the bytemuck bridge.
- **Verified (slice 1, this change):** the native kernel is bit-exact vs `half`
  by exhaustive widen (all 2¹⁶, both formats), exhaustive finite round-trip, a
  ~4.2M-case rounding sweep across every exponent/round/guard/sticky decision,
  and pinned known-value/ties-to-even cases. `fmt`/`clippy -D warnings`/`nextest`
  (52/52)/doctest/rustdoc all clean; purely additive `pub mod convert` ([minor]).
- Follow-ups tracked as [backlog.md](../../backlog.md) E-022…E-030.

[#129097]: https://github.com/rust-lang/rust/issues/129097
