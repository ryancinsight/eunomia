//! Derive macros for Eunomia's native byte-layout marker traits.

#![deny(missing_docs)]

mod expand;

use expand::expand_marker;
use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Derive `eunomia::Zeroable` for a C or transparent representation.
///
/// Every field must already implement `eunomia::Zeroable`. The derive rejects
/// Rust's default representation and packed representations because their
/// layout is not a stable byte contract for GPU or FFI transport.
#[proc_macro_derive(Zeroable)]
pub fn derive_zeroable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    expand_marker(&input, Marker::Zeroable)
}

/// Derive `eunomia::Pod` for a padding-free C or transparent representation.
///
/// Apply [`Zeroable`](derive@Zeroable) as well: `Pod` intentionally retains
/// Eunomia's marker-trait layering, so a type must prove both all-zero
/// validity and arbitrary-byte validity. Generic C representations are
/// rejected because stable Rust cannot prove that arbitrary generic field
/// combinations contain no padding; generic transparent one-field wrappers
/// remain supported.
#[proc_macro_derive(Pod)]
pub fn derive_pod(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    expand_marker(&input, Marker::Pod)
}

#[derive(Clone, Copy)]
pub(crate) enum Marker {
    Zeroable,
    Pod,
}
