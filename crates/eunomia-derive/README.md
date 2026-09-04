# eunomia-derive

`eunomia-derive` provides the procedural derives for Eunomia's native
`Pod` and `Zeroable` byte-layout marker traits. It is re-exported by the
`eunomia` crate; applications normally depend only on `eunomia`.

`Pod` derives include a padding proof for concrete `#[repr(C)]` types. Generic
`#[repr(C)]` types are rejected because stable Rust cannot prove padding-free
layout for arbitrary field combinations; use a one-field `#[repr(transparent)]`
generic wrapper or write a manual implementation with its explicit layout
proof. Generic `Zeroable` derives remain available for C representations.
