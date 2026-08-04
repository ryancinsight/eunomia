# eunomia: The Datatype Law of Atlas

Eunomia is the **datatype law** of the Atlas stack: the single source of
truth for the numeric and scalar datatype vocabulary every other Atlas crate
computes over. It owns the representations — and nothing else.

In myth, Eunomia ("good order") is the daughter of Themis, the *placement
law* of the stack ([`themis`](https://github.com/ryancinsight/themis)
owns NUMA/tier/worker locality). Where themis decides *where* data lives,
eunomia decides *what* data is.

## What eunomia owns

- **Scalar wrapper types** — `F16`, `Bf16`, `F32`, `F64`, `I8`/`I16`/`I32`,
  and sub-byte `F4`/`F8`/`Bf4`/`Bf8` — with exact byte-layout guarantees and
  one native conversion kernel providing exact widening and
  round-to-nearest-ties-to-even narrowing across every reduced format.
- **`Complex<T>`** — the native `re + im·i` vocabulary type that replaces the
  third-party `num_complex::Complex` across the stack.
- **Packed sub-byte formats** — `Packed4`/`PackedBf4`/`PackedF4` storage,
  COW buffers, rkyv archival, and SIMD-accelerated unpack.
- **Conversion lattices** — `CastFrom`/`CastTo`.
- **Element traits** — `NumericElement`, `FloatElement`.
- **Scalar field traits** — `RealField`/`ComplexField` (the `nalgebra`
  scalar-field analogues), so generic numeric code runs over `f32`/`f64` and
  `Complex` without pulling in nalgebra.

## What it does not own

No computation kernels, allocation, scheduling, or backend code. No
vector/matrix/geometry types — those live in `leto` (CPU arrays) and
`hephaestus` (GPU). Execution, SIMD, and allocation belong to `hermes`,
`mnemosyne`, and `moirai`.

## How to read this book

The book teaches the numeric foundation from first principles:

1. **Part I** — the scalar vocabulary: what each IEEE-754 format means, and
   when to choose it;
2. **Part II** — the element and field traits that let generic code run over
   any scalar;
3. **Part III** — conversion and casting: how values move between precisions
   without silent precision loss;
4. **Part IV** — byte layout and packed formats: the reinterpretation
   contract that GPU/FFI boundaries rely on;
5. **Part V** — numeric semantics: relative equality and element operations;
6. **Part VI** — where the crate sits in the Atlas stack and why the
   dependency direction points inward.

Each chapter maps the theory onto the crate's public API, and worked examples
in `examples/` show the abstractions in use. This is the outline edition: the
chapter structure is complete, and the chapters themselves land as DoR items
per subsystem.
