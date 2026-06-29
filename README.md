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
  layout guarantees.
- **`Complex<T>`** — the native `re + im·i` vocabulary type replacing the
  third-party `num_complex::Complex` across the stack.
- **Packed sub-byte formats** — `Packed4`/`PackedBf4`/`PackedF4` storage, COW,
  rkyv archival, and SIMD-accelerated unpack.
- **Conversion lattices** (`CastFrom`/`CastTo`) and **element traits**
  (`NumericElement`/`FloatElement`).
- **GPU datatype vocabulary** (`eunomia-gpu`) — `Vector<T, N>` (`Vec2`/`Vec3`/
  `Vec4`) device-buffer element types in std430/tightly-packed layout.

It owns **no** computation kernels, allocation, scheduling, or backend code —
those belong to hermes (SIMD), mnemosyne (allocation), moirai (execution), and
the domain crates.

## Crates

| Crate | Role |
| --- | --- |
| `eunomia` | Core datatype vocabulary: scalars, complex, packed formats, casts, element traits. `#![no_std]`-capable. |
| `eunomia-gpu` | GPU datatype vocabulary: `repr(C)`/`Pod` vector element types over the eunomia scalars, for device storage/vertex buffers. |

## Position in the Atlas stack

```text
eunomia        datatype law (scalars, complex, packed, gpu vectors)   ← this repo
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

Dependency direction is strictly inward: eunomia depends on nothing Atlas-local
(only `half`, `bytemuck`, `libm`, optional `rkyv`).

## Provenance

The core numeric vocabulary migrated from `hermes-numeric` (which declared
itself the numeric SSOT but was coupled to hermes's SIMD release cadence).
Eunomia promotes it to an independently versioned foundation repo so the stable
datatype vocabulary evolves separately from perf-volatile SIMD kernels. See
[`docs/adr/0001-eunomia-datatype-ssot.md`](docs/adr/0001-eunomia-datatype-ssot.md).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
