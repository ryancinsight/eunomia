//! Derive macros for Eunomia's native byte-layout marker traits.

#![deny(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Error, Meta, Result, Token};

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

/// Derive `eunomia::Pod` for a C or transparent representation.
///
/// Apply [`Zeroable`](derive@Zeroable) as well: `Pod` intentionally retains
/// Eunomia's marker-trait layering, so a type must prove both all-zero
/// validity and arbitrary-byte validity.
#[proc_macro_derive(Pod)]
pub fn derive_pod(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    expand_marker(&input, Marker::Pod)
}

#[derive(Clone, Copy)]
enum Marker {
    Zeroable,
    Pod,
}

fn expand_marker(input: &DeriveInput, marker: Marker) -> TokenStream {
    match expand_marker_impl(input, marker) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_marker_impl(input: &DeriveInput, marker: Marker) -> Result<proc_macro2::TokenStream> {
    validate_representation(input)?;

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        Data::Enum(_) => {
            return Err(Error::new_spanned(
                &input.ident,
                "Eunomia byte-layout markers support structs only",
            ));
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(
                &input.ident,
                "Eunomia byte-layout markers do not support unions",
            ));
        }
    };

    if has_transparent_representation(input) && fields.len() != 1 {
        return Err(Error::new_spanned(
            &input.ident,
            "#[repr(transparent)] requires exactly one field for Eunomia byte-layout markers",
        ));
    }

    let field_types = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    let trait_path = match marker {
        Marker::Zeroable => quote!(::eunomia::Zeroable),
        Marker::Pod => quote!(::eunomia::Pod),
    };

    let mut impl_generics = input.generics.clone();
    {
        let where_clause = impl_generics.make_where_clause();
        for field_type in &field_types {
            where_clause
                .predicates
                .push(parse_quote!(#field_type: #trait_path));
        }
        if matches!(marker, Marker::Pod) {
            let ident = &input.ident;
            let (_, type_generics, _) = input.generics.split_for_impl();
            where_clause
                .predicates
                .push(parse_quote!(#ident #type_generics: ::eunomia::Zeroable));
        }
    }
    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();
    let ident = &input.ident;
    let (_, type_generics, _) = input.generics.split_for_impl();

    let layout_assertion = layout_assertion(input, &field_types);
    let expansion = match marker {
        Marker::Zeroable => quote! {
            #layout_assertion

            // SAFETY: the derive requires a stable C/transparent representation;
            // every field proves all-zero validity through the generated bounds.
            unsafe impl #impl_generics ::eunomia::Zeroable for #ident #type_generics #where_clause {}
        },
        Marker::Pod => quote! {
            #layout_assertion

            // SAFETY: the derive requires a stable C/transparent representation;
            // every field is Eunomia Pod, and the layout assertion rejects padding.
            unsafe impl #impl_generics ::eunomia::Pod for #ident #type_generics #where_clause {}
        },
    };

    Ok(expansion)
}

fn validate_representation(input: &DeriveInput) -> Result<()> {
    let mut has_c = false;
    let mut has_transparent = false;
    let mut has_packed = false;

    for attribute in &input.attrs {
        if !attribute.path().is_ident("repr") {
            continue;
        }
        let representations =
            attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for representation in representations {
            match representation {
                Meta::Path(path) if path.is_ident("C") => has_c = true,
                Meta::Path(path) if path.is_ident("transparent") => has_transparent = true,
                Meta::Path(path) if path.is_ident("packed") => has_packed = true,
                Meta::List(list) if list.path.is_ident("packed") => has_packed = true,
                _ => {}
            }
        }
    }

    if has_packed {
        return Err(Error::new_spanned(
            &input.ident,
            "packed representations cannot safely derive Eunomia byte-layout markers",
        ));
    }
    if has_c == has_transparent {
        return Err(Error::new_spanned(
            &input.ident,
            "Eunomia byte-layout markers require exactly one of #[repr(C)] or #[repr(transparent)]",
        ));
    }
    Ok(())
}

fn has_transparent_representation(input: &DeriveInput) -> bool {
    input.attrs.iter().any(|attribute| {
        attribute.path().is_ident("repr")
            && attribute
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .is_ok_and(|representations| {
                    representations.into_iter().any(|representation| {
                        matches!(representation, Meta::Path(path) if path.is_ident("transparent"))
                    })
                })
    })
}

fn layout_assertion(input: &DeriveInput, field_types: &[&syn::Type]) -> proc_macro2::TokenStream {
    // Generic const expressions cannot yet name arbitrary generic type sizes
    // on the stable toolchain. The field marker bounds remain active for
    // generic types; concrete ABI structs, which are the GPU/FFI use case,
    // receive the complete padding assertion below.
    if !input.generics.params.is_empty() {
        return quote! {};
    }

    let ident = &input.ident;
    let field_sizes = field_types
        .iter()
        .map(|field_type| quote!(::core::mem::size_of::<#field_type>()));
    quote! {
        const _: () = {
            assert!(
                ::core::mem::size_of::<#ident>() == 0usize #( + #field_sizes )*,
                "Eunomia Pod requires a padding-free representation",
            );
        };
    }
}
