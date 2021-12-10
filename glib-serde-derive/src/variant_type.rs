use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;

pub fn impl_variant_type(input: syn::DeriveInput) -> TokenStream {
    let crate_path = super::crate_path();
    let mut repr_attr = None;
    let mut index_attr = None;
    for attr in &input.attrs {
        let is_index = attr.path.is_ident("glib_serde_variant_index");
        let is_repr = attr.path.is_ident("glib_serde_repr");
        if is_index || is_repr {
            if repr_attr.is_some() || index_attr.is_some() {
                abort!(
                    attr,
                    "Only one of #[glib_serde_variant_index] or #[glib_serde_repr] may be specified"
                );
            }
            if is_index {
                index_attr.replace(attr);
            } else if is_repr {
                repr_attr.replace(attr);
            }
        }
    }
    let (static_type, node_type, children) = match &input.data {
        syn::Data::Struct(s) => {
            if let Some(attr) = repr_attr {
                abort!(attr, "#[glib_serde_repr] attribute not allowed on struct");
            }
            if let Some(attr) = index_attr {
                abort!(
                    attr,
                    "#[glib_serde_variant_index] attribute not allowed on struct"
                );
            }
            impl_for_fields(&crate_path, &s.fields)
        }
        syn::Data::Enum(e) => {
            let (tag, tag_str) = repr_attr
                .map(|_| {
                    for attr in &input.attrs {
                        if attr.path.is_ident("repr") {
                            abort!(attr, "#[glib_serde_repr] cannot be used with #[repr]");
                        }
                    }
                    (quote! { INT32 }, "i")
                })
                .or_else(|| index_attr.map(|_| (quote! { UINT32 }, "u")))
                .unwrap_or_else(|| (quote! { STRING }, "s"));
            let tag = quote! { #crate_path::glib::VariantTy::#tag };
            let has_data = e
                .variants
                .iter()
                .any(|v| !matches!(v.fields, syn::Fields::Unit));
            if has_data {
                let static_type_str = format!("({}v)", tag_str);
                let children = e.variants.iter().map(|variant| {
                    let (_, node_type, children) = impl_for_fields(&crate_path, &variant.fields);
                    quote! { #crate_path::VariantTypeNode::new(#node_type, #children) }
                });
                (
                    impl_lazy(
                        &crate_path,
                        quote! { #crate_path::glib::VariantTy },
                        quote! {
                            unsafe {
                                #crate_path::glib::VariantTy::from_str_unchecked(#static_type_str)
                            }
                        },
                    ),
                    quote! {
                        <Self as #crate_path::glib::StaticVariantType>::static_variant_type()
                    },
                    quote! { [ #(#children),* ] },
                )
            } else {
                (
                    quote! { ::std::borrow::Cow::Borrowed(#tag) },
                    quote! { ::std::borrow::Cow::Borrowed(#tag) },
                    quote! { &[] },
                )
            }
        }
        syn::Data::Union(u) => {
            abort!(
                u.union_token,
                "#[derive(glib_serde::VariantType)] is not available for unions."
            );
        }
    };
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let variant_type = impl_lazy(
        &crate_path,
        quote! { #crate_path::VariantTypeNode },
        quote! { #crate_path::VariantTypeNode::new(#node_type, #children) },
    );

    quote! {
        impl #impl_generics #crate_path::glib::StaticVariantType for #name #ty_generics #where_clause {
            fn static_variant_type() -> ::std::borrow::Cow<'static, #crate_path::glib::VariantTy> {
                #static_type
            }
        }

        impl #impl_generics #crate_path::VariantType for #name #ty_generics #where_clause {
            fn variant_type() -> ::std::borrow::Cow<'static, #crate_path::VariantTypeNode<'static>> {
                #variant_type
            }
        }
    }
}

fn impl_for_fields(
    crate_path: &TokenStream,
    fields: &syn::Fields,
) -> (TokenStream, TokenStream, TokenStream) {
    match fields {
        syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
            let types = fields.iter().map(|f| &f.ty);
            let (static_type, node_type) = if fields.len() == 1 {
                let ty = &fields.iter().next().unwrap().ty;
                (
                    quote! {
                        <#ty as glib::StaticVariantType>::static_variant_type()
                    },
                    quote! {
                        <#ty as #crate_path::VariantType>::variant_type().type_()
                    },
                )
            } else {
                let types = types.clone();
                (
                    impl_lazy(
                        crate_path,
                        quote! { #crate_path::glib::VariantType },
                        quote! {
                            {
                                let mut builder = #crate_path::glib::GStringBuilder::new("(");
                                #(
                                    {
                                        let typ = <#types as glib::StaticVariantType>::static_variant_type();
                                        builder.append(typ.as_str());
                                    }
                                 )*
                                builder.append_c(')');

                                #crate_path::glib::VariantType::from_string(builder.into_string()).unwrap()
                            }
                        },
                    ),
                    quote! {
                        <Self as #crate_path::glib::StaticVariantType>::static_variant_type()
                    },
                )
            };
            (
                static_type,
                node_type,
                quote! {
                    [
                        #(
                            <#types as #crate_path::VariantType>::variant_type()
                         ),*
                    ]
                },
            )
        }
        syn::Fields::Unit => (
            quote! { ::std::borrow::Cow::Borrowed(#crate_path::glib::VariantTy::UNIT) },
            quote! { ::std::borrow::Cow::Borrowed(#crate_path::glib::VariantTy::UNIT) },
            quote! { [] },
        ),
    }
}

fn impl_lazy(crate_path: &TokenStream, ty: TokenStream, value: TokenStream) -> TokenStream {
    quote! {
        static TYP: #crate_path::glib::once_cell::sync::Lazy<#ty>
            = #crate_path::glib::once_cell::sync::Lazy::new(|| #value);
        ::std::borrow::Cow::Borrowed(&*TYP)
    }
}
