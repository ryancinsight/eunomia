# 11. Packed Sub-byte Formats

## Governing equations

The sub-byte float formats (`F4` E3M0, `F8` E4M3, `Bf4` E2M1, `Bf8` E5M2)
are stored one per `u8`, wasting 4–7 bits per element. A *packed* layout
packs multiple sub-byte values into one storage word — two 4-bit values per
`u8` — trading a little unpack arithmetic for a 2× reduction in memory
footprint and bandwidth, which matters at scale for the storage layers of
the stack (GPU uploads, archives).

## The crate's abstraction

The `packed` module provides the storage vocabulary over these formats:

- **Buffers** — `Packed4Vec`/`PackedBf4Vec`/`PackedF4Vec` and
  `Packed4Slice`/`Packed4SliceMut` (+ per-format slices), with `Packed4Iter`.
- **COW** — `Packed4Cow`/`PackedBf4Cow`/`PackedF4Cow` for
  copy-on-write buffers that avoid copying until mutation.
- **Archival** — rkyv support (`ArchivedPacked4Cow`, `ArchivedPacked4Vec`,
  resolvers) under the `rkyv` feature, so packed buffers serialize without
  an allocation.
- **SIMD-accelerated unpack** — `unpack_f4_to_f32`, `unpack_bf4_to_bf16`,
  `unpack_f8_to_f32`, etc., with `unsafe_intrinsics` on `x86_64`/`aarch64`.

```rust,ignore
use eunomia::packed::{unpack_f4_to_f32, Packed4Vec, Packable4};
```

## Outline of this chapter

- Why pack: sub-byte formats and the 2× footprint win
- The buffer vocabulary: `Vec`/`Slice`/`Iter` and the `Packable4` trait
- COW buffers and when copy-on-write earns its keep
- rkyv archival of packed buffers under `no_std`
- SIMD-accelerated unpack and the `unsafe_intrinsics` gate
- Storage boundaries: packed on disk/device, unpacked in registers
