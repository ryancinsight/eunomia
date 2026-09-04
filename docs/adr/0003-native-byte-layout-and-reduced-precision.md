# ADR 0003: Native byte-layout & reduced-precision vocabulary

- Status: Accepted (native conversion and runtime dependency retirement landed)
- Date: 2026-07-18
- Class: [arch] (grows eunomia's owned surface; shrinks external deps; ripples to
  every reduced-precision / GPU-byte consumer)
- Supersedes/relates: [ADR 0001](0001-eunomia-datatype-ssot.md) (datatype SSOT),
  [gap_audit.md](../../gap_audit.md) §Byte-layout / reduced-precision.

## Context

Eunomia is the datatype law but does not fully own two datatype concerns it is
the natural home for:

1. **Reduced precision.** `F16`/`Bf16` previously wrapped the external
   `half::f16`/`half::bf16`; the provider did not own their representation. The sub-byte
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
- **The external reduced-precision implementation has no required lock-in**;
  its stack surface was a bounded set of conversion methods and public element
  types. It is fully replaceable by provider-owned representations.
- The stack's entire `zerocopy` need is one call (`IntoBytes::as_bytes`, in
  out-of-scope consus). The earlier external reduced-precision implementation
  supplied a differential oracle, but it is not required by Eunomia's native
  implementation or test source.

## Decision

**D1 — Own reduced-precision conversion natively; retire the `half` runtime
dependency.** One generic const-parameterized IEEE-754 narrow/widen kernel
(`convert::{narrow, widen}`, `<const E, const M>`, round-to-nearest-ties-to-even,
subnormals, inf/NaN, f32-subnormal handling) is the single conversion SSOT.
`binary16` (`E=5,M=10`) and `bfloat16` (`E=8,M=7`) instantiate it; the sub-byte
formats fold onto it, deleting the four hand-rolled copies and the truncation
defect. The implementation has no external reduced-precision dependency; its
integration tests use an independent IEEE-754 value-level oracle. The working
branch is the consumer-migration boundary; no interop feature or bridge trait
implementation merges into the default branch.

**D2 — Own a byte-layout vocabulary at zerocopy's checked tier; bridge, do not
replace, bytemuck.** Eunomia gains marker traits (`Zeroable`, `Pod`-equivalent)
and the safe reinterpretation the stack actually uses (`cast_slice`(+mut),
`bytes_of`, `from_bytes`, unaligned read), machine-checked for eunomia's own
types via the existing `const _` size/align assertions + per-impl `// SAFETY:`.
Because `bytemuck` remains a mandatory dependency for the GPU boundary, the
interop is unconditional rather than feature-gated: Eunomia-owned wrappers and
`Complex<T>` receive co-located implementations of both Eunomia's markers and
`bytemuck::{Pod,Zeroable}`. A blanket bridge is not possible under Rust's orphan
rules. The re-exported derives support concrete `repr(C)` and one-field
`repr(transparent)` ABI types; generic `repr(C)` `Pod` derives are rejected
because stable Rust cannot prove that arbitrary field combinations contain no
padding. Backend-owned ABI structs continue to own their direct `bytemuck`
contracts. Scope is bounded to what the audit found consumers use — **no
OCP/checked-transmute surface is built speculatively.**

**D3 — Pin the sub-byte special-value convention explicitly; add OCP-MXFP as a
distinct format family only when a consumer needs it.** `Bf8` (E5M2) uses the
IEEE infinity/NaN convention. `F8` (E4M3), `Bf4` (E2M1), and `F4` (E3M0) are
finite-only: the whole top exponent is reserved for NaN, and narrowing
saturates infinity or overflow to the signed maximum finite value. These
contracts are documented and reference-tested. OCP-MXFP FP8/FP4 (no infinity;
the emerging GPU-quantization standard, which Eunomia's `F4`=E3M0 matches no
format of) is added as new types selected through a public special-value policy
parameter **when Coeus/Hephaestus quantization requires it** — not before.

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

- Eunomia's production and test source contains no external reduced-precision
  provider; the datatype law owns both the representations and conversions,
  while tests use an independent IEEE-754 oracle. Criterion's benchmark-only
  dependency graph still resolves `half` transitively through `ciborium`; that
  upstream serializer edge is not imported or used by Eunomia's datatype code.
- One conversion kernel replaces five implementations and fixes the
  truncation/convention defects (G-C2/G-C3/G-A3).
- Hermes, Leto, Apollo, and other Atlas consumers use `eunomia::F16`/`Bf16`
  directly. Apollo's compact complex FFT surface uses `Complex<F16>`; no
  consumer retains the external `half::f16` as a production representation.
- **Verified (slice 1, this change):** the native kernel matches the independent
  IEEE-754 reference for finite, infinite, zero, and NaN value-class semantics by exhaustive widen (all
  2¹⁶ patterns, both formats), exhaustive finite round-trip, a ~4.2M-case
  rounding sweep across every exponent/round/guard/sticky decision, and pinned
  known-value/ties-to-even cases. NaN payload bits are intentionally not a
  contract. `fmt`/`clippy -D warnings`/`nextest` (52/52)/doctest/rustdoc all
  clean; purely additive `pub mod convert` ([minor]).
- **Verified (slice 2):** E5M2, E2M1, E4M3, and E3M0 now instantiate the same
  kernel through monomorphized IEEE or finite-only policies. Analytical
  known-value, special-value, exhaustive finite-encoding round-trip, and
  ties-to-even tests pin the four contracts. Exhaustive packed-dispatch
  differential tests pin runtime-selected table wiring and directly exercise
  AVX2/AVX-512 when the host reports those capabilities. The cutover also
  corrects the E5M2/E2M1 subnormal scales and four-bit finite-limit/sign
  constants. A dependency-free AArch64 compile harness includes the actual
  kernel and NEON module, providing compile-time verification of that ISA path.
- Follow-ups tracked as [backlog.md](../../backlog.md) E-022…E-030.

## Revision note — 2026-09-02

The original decision allowed Apollo's raw `half::f16` FFT surface to remain
consumer-owned. The stack contract is now clarified: Eunomia owns reduced-
precision scalar and complex representations, and Apollo's compact route is a
consumer migration to `F16`/`Complex<F16>`. The provider implementation and
its independent IEEE-754 test oracle are unchanged.

The 2026-09-03 oracle cleanup removed Eunomia's direct `half` declarations and
imports. The development tests now use the independent IEEE-754 reference
module; the Criterion serializer's unrelated transitive edge remains visible
in `Cargo.lock` and is not a datatype-provider contract.

The byte-layout implementation reconciles the planned interop feature with the
current dependency contract. Dual marker implementations now live in
`layout::marker`, while backend-owned ABI types remain direct `bytemuck`
contracts. Checked slice-length arithmetic rejects source-byte-count overflow,
and fallible unaligned reads expose short input as `PodCastError` instead of
requiring a panic at an untrusted boundary.

[#129097]: https://github.com/rust-lang/rust/issues/129097
