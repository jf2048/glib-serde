// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

mod enums;
mod variant_type;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

fn crate_path() -> proc_macro2::TokenStream {
    use proc_macro_crate::{crate_name, FoundCrate};

    if let Ok(FoundCrate::Name(name)) = crate_name("glib_serde") {
        let ident = quote::format_ident!("{}", name);
        quote::quote! { ::#ident }
    } else {
        quote::quote! { ::glib_serde }
    }
}

/// Generates `VariantType` trait so this type can be serialized. Supports structs and enums.
#[proc_macro_derive(VariantType, attributes(glib_serde_variant_index))]
#[proc_macro_error]
pub fn variant_type_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    variant_type::impl_variant_type(input).into()
}

/// Implements `serde::Deserialize` for types using `#[derive(glib::Enum)]`.
#[proc_macro_derive(EnumDeserialize, attributes(glib_serde_repr))]
#[proc_macro_error]
pub fn enum_deserialize_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    enums::impl_enum_deserialize(input).into()
}

/// Implements `serde::Serialize` for types using `#[derive(glib::Enum)]`.
#[proc_macro_derive(EnumSerialize, attributes(glib_serde_repr))]
#[proc_macro_error]
pub fn enum_serialize_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    enums::impl_enum_serialize(input).into()
}

/// Implements `serde::Deserialize` for types using `#[glib::flags]`.
#[proc_macro_derive(FlagsDeserialize, attributes(glib_serde_repr))]
#[proc_macro_error]
pub fn flags_deserialize_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    enums::impl_flags_deserialize(input).into()
}

/// Implements `serde::Serialize` for types using `#[glib::flags]`.
#[proc_macro_derive(FlagsSerialize, attributes(glib_serde_repr))]
#[proc_macro_error]
pub fn flags_serialize_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    enums::impl_flags_serialize(input).into()
}
