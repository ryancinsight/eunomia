# ADR 0007: Retire CastFrom and CastTo

- Status: Proposed
- Date: 2026-10-02
- Item: EUNOMIA-CASTFROM-RETIRE (delivered by this ADR's PR, which deletes the
  board entry)
- Stack contract: atlas ADR 0005 (bare `as` conversions)

## Context

`CastFrom<T>` was a generic `as` between any pair of primitives, and `CastTo`
was its blanket reverse spelling. `src/casts/primitives.rs` expanded one
macro into roughly 140 `val as $dst` impls. `src/casts/wrappers.rs` routed
every `F*`/`Bf*` wrapper pair through `from_f64(to_f64())`, and every integer
wrapper pair through `Self(val.0 as _)`. A call site named neither the
conversion's contract (exact, rounded, saturated, truncated or wrapped) nor
its failure mode. The stack cast rule bans exactly this helper class: a
conversion helper outside the one conversion module that wraps `as` and does
not state what enters it.

`NumericElement` carried `CastFrom<i32>` as a supertrait. Inside eunomia, only
`Complex<T>`'s own `CastFrom<i32>` impl consumed it. Downstream, generic code
used `T::cast_from(n)` to build an element from a length or a small constant.

At the time of this decision, eunomia's own tree held 89 sites matching the
retirement grep (87 code sites and two doc-comment lines in
`convert/count.rs`). Consumer members migrate in their own items; apollo and
helios have landed.

## Decision

Delete `CastFrom`, `CastTo`, `traits/cast.rs` and the `casts/` module. Drop
the `NumericElement: CastFrom<i32>` supertrait and the
`Complex<T>: CastFrom<i32>` impl. Every former use maps onto a conversion
that states its contract. Float into integer is the one direction that had
no named conversion, so it landed first as
`convert::IntegerTarget` (PR #157):

| Old form | Replacement |
| --- | --- |
| lossless int widening (`u32::cast_from(u8)`) | `From` / `.into()` |
| int narrowing (`i32::cast_from(i64)`) | `TryFrom`, with the failure handled |
| same-width sign reinterpretation (`u8::cast_from(-1_i8)`) | `cast_unsigned()` / `cast_signed()` |
| count into a generic element (`T::cast_from(len as i32)`) | `TryFromCount::try_from_count(len)` (exact, or a typed `CountRangeError`) |
| count into a float element | `FloatElement::from_count` (rounds to nearest, ties to even) |
| signed int into a float element | `FloatElement::from_integer(i64)` |
| element into `f64` (`f64::cast_from(x)`) | `NumericElement::to_f64` |
| `f64` into a float element | `FloatElement::from_f64` |
| `f32` into `f64` | `f64::from` |
| float format to float format (`F32::cast_from(F16)`) | `from_f64(x.to_f64())`, or the native kernel `convert::{narrow, widen}` |
| float into integer, rounded (`u8::cast_from(x.round())`) | `IntegerTarget::from_rounded(x)` (nearest, ties to even) |
| float into integer, floored (`usize::cast_from(x.floor())`) | `IntegerTarget::from_floor(x)` |
| float into integer, truncated (`usize::cast_from(x)`) | `IntegerTarget::from_truncated(x.to_f64())` |
| integer wrapper to or from its primitive | `.0` / `Self(..)` |

The `IntegerTarget` conversions saturate (NaN gives 0), matching what the
retired `as` casts did, so a consumer migrates without a behavior change. A
consumer that needs out-of-range values reported instead keeps an explicit
range check, or later adds a checked method to `convert/` that returns a
typed error.

## Rejected alternatives

- **Keep `CastFrom` and narrow it to lossless pairs.** That re-creates `From`
  under another name, and the narrowing and float pairs consumers rely on
  would still need named contracts, so the trait would remain a second
  conversion vocabulary.
- **Put the float-to-integer conversions on `FloatElement`.** That trait's
  file was already near the 500-line target. An `IntegerTarget` method names
  its result type at the call site (`u8::from_rounded(x)`) the way
  `TryFrom` does.
- **Keep the `CastFrom<i32>` supertrait as a named `from_small_integer`
  method on `NumericElement`.** Every consumer use was a length, a count, or a
  float constant, and `try_from_count`, `from_count` and `from_integer` already
  cover those with stated contracts. A new method would duplicate them.
- **Deprecate first, delete later.** A `#[deprecated]` bridge is the
  compatibility layer the stack rule forbids. Consumers migrate before this
  deletion merges instead (see Consequences).

## Consequences

- Breaking change, [major]. `eunomia::{CastFrom, CastTo}` no longer exist, and
  `NumericElement` no longer implies `CastFrom<i32>`. The migration guide is
  the replacement table above, carried in the commit's `BREAKING CHANGE:`
  footer.
- The delivering PR stays draft until
  `git grep -E '\b(cast_from|cast_to|CastFrom|CastTo)\b' -- '*.rs'` is empty
  on every stack member's default branch. The stack therefore never builds
  against a missing trait.
- About 140 bare `as` impls leave the crate, and `tests/cast.rs` (which tested
  `as` semantics through the trait) deletes. The integer and float element
  contract tests now assert `try_from_count` and `from_integer` in place of
  `cast_from`.
- Book chapter 8 now teaches the conversion surface: std traits, the
  conversion module, and the `from_f64`/`to_f64` boundary.

## Overturning evidence

A consumer pattern that the table cannot express without a bare `as` would
reopen this decision, for example a generic kernel that needs a signed,
non-count integer in a non-float `NumericElement`. That case adds one
contract-stating method to the conversion module; it never restores a
pair-generic trait.
