# Eunomia

Eunomia is the **datatype law** of the Atlas stack: the single source of truth
for numeric and scalar datatype *vocabulary*. It owns the representations every
other Atlas crate computes over — and nothing else.

It is the sibling of [`themis`](https://github.com/ryancinsight/themis)
(*placement law* — NUMA/tier/worker locality). Where themis supplies the law of
*where* data lives, Eunomia supplies the law of *what* data is. In myth, Eunomia
("good order") is Themis's daughter; here the datatype law sits one rung below
the placement law in the foundation.

## What it owns

- **Scalar wrapper types** — `F16`, `Bf16`, `F32`, `F64`, `I8`/`I16`/`I32`, and
  sub-byte `F4`/`F8`/`Bf4`/`Bf8` — with `bytemuck::Pod`/`Zeroable` and exact
  layout guarantees. One native conversion kernel provides exact widening and
  round-to-nearest, ties-to-even narrowing across every reduced format.
- **`Complex<T>`** — the native `re + im·i` vocabulary type replacing the
  third-party `num_complex::Complex` across the stack.
- **Packed sub-byte formats** — `Packed4`/`PackedBf4`/`PackedF4` storage, COW,
  rkyv archival, and SIMD-accelerated unpack.
- **Conversion lattices** (`CastFrom`/`CastTo`) and **element traits**
  (`NumericElement`/`FloatElement`).
- **Scalar field traits** — `RealField`/`ComplexField` (the `nalgebra` scalar
  field analogues), so generic numeric code runs over `f32`/`f64` and `Complex`.

It owns **no** computation kernels, allocation, scheduling, or backend code, and
**no vector/matrix/geometry types** — those are linear algebra and live in leto
(CPU) / hephaestus (GPU). The other layers belong to hermes (SIMD), mnemosyne
(allocation), moirai (execution), and the domain crates.

## Crate

| Crate | Role |
| --- | --- |
| `eunomia` | The datatype vocabulary: scalars, complex, packed formats, casts, element + scalar-field traits. `#![no_std]`-capable. |

## Position in the Atlas stack

```text
eunomia        datatype law (scalars, complex, packed, field traits)   ← this repo
themis         placement law (NUMA, tier, worker locality)
   ↑ consumed by
hermes         SIMD execution over eunomia types
mnemosyne      allocation
moirai         execution
   ↑ consumed by
leto           array substrate (cache-tiled over hermes/eunomia)
   ↑ consumed by
coeus / hephaestus / apollo / …   domain
```

Dependency direction is strictly inward: eunomia depends on nothing Atlas-local.
Its core uses `half`, `bytemuck`, and `libm`; `rkyv`, `serde`, and the
NumPy/PyO3 0.29 element boundary are optional.

## Provenance

The core numeric vocabulary migrated from `hermes-numeric` (which declared
itself the numeric SSOT but was coupled to hermes's SIMD release cadence).
Eunomia promotes it to an independently versioned foundation repo so the stable
datatype vocabulary evolves separately from perf-volatile SIMD kernels. See
[`docs/adr/0001-eunomia-datatype-ssot.md`](docs/adr/0001-eunomia-datatype-ssot.md).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
