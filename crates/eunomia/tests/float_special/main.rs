//! Value-semantic contract tests for the `FloatElement` special functions
//! (`log10`, `log2`, `erf`, `erfc`, `lgamma`, plus the ADR-0061-required
//! `asin`/`atan`/`acosh`/`asinh`/`atanh`/`exp2`/`exp_m1`/`ln_1p`),
//! cross-checked against analytic references rather than asserting mere
//! existence.
//!
//! Calls use fully-qualified `FloatElement::…` syntax: std is stabilizing
//! same-named inherent float methods (`unstable_name_collisions`), and method
//! syntax would silently rebind to those, changing which implementation the
//! test verifies. Qualification pins the trait under test.
//!
//! The ADR-0061 additions test two genuinely different routes, each with its
//! own correct oracle (never std's float methods, which are an independent
//! implementation with no derivable error bound relative to `libm`, and whose
//! *input* conditioning near a pole — `atanh` approaching ±1 — further
//! amplifies any inter-implementation disagreement to hundreds of ulp):
//! - `f64`/`F64` call `libm`'s `f64` entry point directly, so the correct
//!   assertion is bitwise equality against that exact same call (wiring, not
//!   accuracy) — see `new_transcendentals_f64_and_f64_wrapper_wire_to_libm_bitwise`.
//! - `f32` routes through `libm`'s independent `f32` algorithm, so it is
//!   compared against `libm`'s `f64` result rounded to `f32` (an oracle with
//!   zero input error, since every `f32` widens to `f64` exactly), within a
//!   bound derived from `libm`'s own documented per-function error where the
//!   source states one — see
//!   `new_transcendentals_f32_match_libm_f64_oracle_within_documented_bound`.

mod domain_edges;
mod fixtures;
mod references;
mod transcendentals;
