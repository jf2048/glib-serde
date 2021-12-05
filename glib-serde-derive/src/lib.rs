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

#[proc_macro_derive(VariantType, attributes(glib_serde_repr))]
#[proc_macro_error]
pub fn variant_type_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    variant_type::impl_variant_type(input).into()
}
