# ADR 0013: Real components for Complex NumericElement

Status: Accepted  \
Date: 2026-11-24  \
Change class: [major] (narrows the bound of a public impl)  \
Delivery: EUNOMIA-COMPLEX-NESTED-ELEMENT (eunomia#229)

Revision 2026-11-24: Accepted with its delivery; the marker covers all 23
non-`Complex` element types (was 18), and the nested-`sqrt` witness, the
in-crate helper bound and the consumer wording are corrected.

## Context

`impl<T> NumericElement for Complex<T>` bounded `T: NumericElement + Neg`
(`crates/eunomia/src/impls/primitives/numeric.rs:247-250` at `fa17b38`).
`Complex<T>` is itself `NumericElement`, so `Complex<Complex<f64>>` satisfied
the bound.

`Complex::min_scalar` and `max_scalar` order by `re`, then `im`, with `<`, `>`,
`<=` and `>=` (`numeric.rs:359-374`). `PartialOrd for Complex` returns `None`
for every pair (`types/complex/ops.rs:256-262`), so on a `Complex` component all
four comparisons are false and both functions return their second argument.

Reproduced at `e9dbcf8` (tree of this branch's base): adding
`complex_reduction_identity_failures::<Complex64>()` after the `I32` row of
`complex_reduction_identity_returns_the_element`
(`tests/complex_provider_contract.rs:130-150`) and running
`cargo nextest run -p eunomia --test complex_provider_contract` fails that test
(exit 100) with 28 failing cells, all `max seed right` or `min seed right`:
`max_scalar(element, MIN_VALUE)` returns `MIN_VALUE`. The test file was
restored byte for byte afterwards.

The same impl's other members use ordered components only in `sqrt`
(`self.im < ZERO`, `numeric.rs:317`) and in `min_scalar`/`max_scalar`. The
operators (`types/complex/ops.rs`), `TryFromCount` (`convert/count.rs:261`),
`Sum`/`Product` (`types/complex/reduce.rs`) and `ComplexField for Complex<T>`
(`impls/field.rs:208`, `T: RealField`) do not need an ordered component and are
unchanged by any option below.

Component types the contract test exercises today (`complex_provider_contract.rs:132-149`):
`f32`, `f64`, `i8`, `i16`, `i32`, `i64`, `isize`, `F16`, `Bf16`, `F32`, `F64`,
`Bf8`, `Bf4`, `F8`, `F4`, `I8`, `I16`, `I32` (18 types). The five unsigned
primitives are `NumericElement` too and are not in the grid: they have no
`Neg`, which the `Complex` impl also requires.

### In-stack use (read at each member's fetched default branch, 2026-11-24)

`git grep -n -E 'Complex<\s*Complex'` over all 28 members registered in
`.gitmodules` finds no code instantiating a nested complex. Eunomia matches
only in `backlog.md` (6 prose lines). The grep is line-based and misses a
nested form split across lines or reached through a type alias.

Generic code that makes `Complex<T>` a `NumericElement` for a caller-chosen `T`
does exist:

- leto `crates/leto-ops/src/domain/scalar/impls.rs:313`:
  `impl<T> Scalar for Complex<T> where T: Scalar + Neg<Output = T>`, with
  `Scalar: NumericElement` (`domain/scalar/contract.rs:18`). `T: Scalar`
  admits integers and `Complex` itself, so `Complex<Complex<f32>>` is
  `leto::Scalar` today and the bound does not imply any real-component bound.
- coeus `crates/coeus-core/src/dtype/complex.rs:165`:
  `impl<T: Float + FloatElement + Neg<Output = T>> Scalar for Complex<T>`, with
  `Scalar: NumericElement` (`dtype/traits/scalar.rs:42-43`); `T: FloatElement`
  is never complex.

Apollo uses `Complex<F16>` (for example `apollo-fft/src/api/cfft.rs:52`) and
eunomia pins `Complex<F16>` and `Complex<Bf16>` layout (`types/mod.rs:20-28`);
both stay admitted by the decision.

### Existing real-scalar traits do not fit

- `RealField` has four impls: `f32`, `f64`, `F32`, `F64`
  (`impls/field.rs:12,39,75,104`). It would remove `F16`, `Bf16`, the 8-bit and
  4-bit formats and every integer component.
- `FloatElement` has no integer impl (integer impls are `NumericElement` only:
  `impls/wrappers/numeric.rs:318,355,392`, `impls/primitives/numeric.rs:241-245`).
  It would remove the 8 integer rows of the contract test.
- `PartialOrd` does not separate the cases: `Complex` implements it.

## Decision

Restrict the impl to real components with one sealed marker trait,
working name `RealElement`, in `traits/`:

1. `impl<T: RealElement + Neg<Output = T>> NumericElement for Complex<T>`.
   `RealElement: NumericElement` is implemented by every non-`Complex`
   `NumericElement` type (the 18 above and `u8`, `u16`, `u32`, `u64`,
   `usize`: 23) and by no `Complex`. The unsigned five carry it so the name
   needs no exception list; `Neg` still excludes them as components, so the
   admitted set stays the 18. Sealing comes from the `private::Sealed`
   supertrait of `NumericElement` (`traits/numeric.rs:16-17`).
2. `FloatElement` gains `RealElement` as a supertrait, so every
   `T: FloatElement` caller (coeus above) keeps the impl without edits. The
   integer types implement the marker directly.
3. A `compile_fail,E0277` doctest on the impl shows
   `Complex<Complex<f64>>: NumericElement` rejected; a positive doctest shows
   `Complex<f64>` and `Complex<f32>` accepted; the grid test above gains no
   row, because no admitted component changes. Its helper bound
   (`tests/complex_provider_contract.rs:84`, `T: NumericElement + Neg`) takes
   `RealElement` in the same commit, and a value test pins the
   `MIN_VALUE`/`MAX_VALUE` folds of `Complex<f64>`.
4. The release that carries the change ships with leto `impls.rs:313` adding
   the marker to its bound, in one co-evolution unit (architecture_scoping:
   co-evolution protocol).

## Alternatives rejected

- Give `Complex<Complex<T>>` a total lexicographic order so the existing
  `MIN_VALUE`/`MAX_VALUE` fold identities hold. `PartialOrd for Complex` is
  `None` by design (`ops.rs:256-262`) and ADR 0005 states `Complex<T>` is not a
  real scalar; ordering the nested case would add an order to the one type that
  must not have one, to serve an instantiation with no in-stack use.
- Bound the impl on `RealField` or `FloatElement`: each removes tested,
  shipped component rows (see Context).
- Keep the bound and document that nested complex reduces wrongly: leaves a
  safe-code wrong result on a public contract (the 28 cells above).
- Fix only `min_scalar`/`max_scalar` to recurse through the component's own
  `min_scalar`: this repairs the folds without a bound change (the recursion
  yields the documented lexicographic order), but nested `sqrt`
  (`numeric.rs:311-321`) stays wrong and no order-based sign rule repairs
  it: `(0, -2+0i)` gives `(1, 1)`, whose square is `(0, 2)` (derived by hand
  from the source, not run), and `im = -1+2i` needs `v` kept at
  `re = -2-2i` and negated at `re = 0` (principal roots, checked with a
  Python `cmath` model), so a sign rule that reads only `im` cannot tell the
  two apart. Narrowing now can be widened later without a break; keeping the
  nested impl cannot be withdrawn without one.

## Consequences

- `Complex<Complex<T>>` stops being `NumericElement`; code that spelled it stops
  compiling (E0277) and must use a real component. No in-stack code is known to
  spell it.
- Generic code that relies on the impl under
  `T: NumericElement + Neg<Output = T>` (or a trait whose only supertrait is
  `NumericElement`, such as leto `Scalar`) stops compiling until `T` gains the
  marker; code bounded only `T: NumericElement` never had the impl. Known
  site: leto `impls.rs:313`. Derived from the two cited bounds; leto is not
  compiled against the change.
- Cargo treats 0.12 to 0.13 as breaking; the version is chosen at release,
  which this ADR does not authorize.
- One new public sealed trait and one new supertrait on `FloatElement`.

## Overturning evidence

A consumer that needs a nested or quaternion-style `Complex<Complex<T>>` as a
`NumericElement` (the decision then becomes an order for that type), a
real-component type that carries no marker, or a leto migration that cannot add
the marker bound without dropping a real `T` it ships.
