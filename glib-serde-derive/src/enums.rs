use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;

fn get_glib_serde_repr_attr(input: &syn::DeriveInput) -> Option<&syn::Attribute> {
    let mut repr_attr = None;
    for attr in &input.attrs {
        let is_repr = attr.path.is_ident("glib_serde_repr");
        if is_repr {
            if repr_attr.is_some() {
                abort!(attr, "Only one of #[glib_serde_repr] may be specified");
            }
            repr_attr.replace(attr);
        }
    }
    repr_attr
}

fn get_repr_attr(input: &syn::DeriveInput) -> Option<&syn::Attribute> {
    for attr in &input.attrs {
        if attr.path.is_ident("repr") {
            return Some(attr);
        }
    }
    None
}

pub fn impl_enum_serialize(input: syn::DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let crate_path = super::crate_path();
    match &input.data {
        syn::Data::Struct(s) => {
            abort!(
                s.struct_token,
                "#[derive(glib_serde::EnumSerialize)] is not available for structs"
            );
        }
        syn::Data::Union(u) => {
            abort!(
                u.union_token,
                "#[derive(glib_serde::EnumSerialize)] is not available for unions"
            );
        }
        syn::Data::Enum(_) => (),
    };
    let repr_attr = get_glib_serde_repr_attr(&input);

    let (getter, serialize) = if repr_attr.is_some() {
        if let Some(attr) = get_repr_attr(&input) {
            abort!(attr, "#[glib_serde_repr] cannot be used with #[repr]");
        }

        (quote! { value }, quote! { serialize_i32 })
    } else {
        (quote! { nick }, quote! { serialize_str })
    };

    quote! {
        impl #crate_path::serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: #crate_path::serde::Serializer
            {
                let ty = <Self as #crate_path::glib::StaticType>::static_type();

                let enum_class = match #crate_path::glib::EnumClass::new(ty) {
                    ::std::option::Option::Some(enum_class) => enum_class,
                    ::std::option::Option::None => {
                        return ::std::result::Result::Err(
                            #crate_path::serde::ser::Error::custom(
                                format!("No such enum {}", ty)
                            )
                        )
                    }
                };
                let value = <Self as #crate_path::glib::translate::IntoGlib>::into_glib(*self);
                let enum_value = match #crate_path::glib::EnumClass::value(&enum_class, value) {
                    ::std::option::Option::Some(value) => value,
                    ::std::option::Option::None => {
                        return ::std::result::Result::Err(
                            #crate_path::serde::ser::Error::custom(
                                format!("Invalid enum value `{}` for {}", value, ty)
                            )
                        )
                    }
                };
                serializer.#serialize(enum_value.#getter())
            }
        }
    }
}

pub fn impl_enum_deserialize(input: syn::DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let crate_path = super::crate_path();
    match &input.data {
        syn::Data::Struct(s) => {
            abort!(
                s.struct_token,
                "#[derive(glib_serde::EnumDeserialize)] is not available for structs"
            );
        }
        syn::Data::Union(u) => {
            abort!(
                u.union_token,
                "#[derive(glib_serde::EnumDeserialize)] is not available for unions"
            );
        }
        syn::Data::Enum(_) => (),
    };
    let repr_attr = get_glib_serde_repr_attr(&input);

    let deserialize = if repr_attr.is_some() {
        if let Some(attr) = get_repr_attr(&input) {
            abort!(attr, "#[glib_serde_repr] cannot be used with #[repr]");
        }

        quote! { deserialize_i32 }
    } else {
        quote! { deserialize_str }
    };

    quote! {
        impl<'de> #crate_path::serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #crate_path::serde::Deserializer<'de>,
            {
                struct EnumVisitor(#crate_path::glib::EnumClass);

                impl<'de> #crate_path::serde::de::Visitor<'de> for EnumVisitor {
                    type Value = #ident;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(
                            formatter,
                            "a valid enum value for {}",
                            &#crate_path::glib::EnumClass::type_(&self.0)
                        )
                    }

                    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
                    where
                        E: #crate_path::serde::de::Error,
                    {
                        match #crate_path::glib::EnumClass::value(&self.0, v) {
                            ::std::option::Option::Some(value) => {
                                ::std::result::Result::Ok(
                                    unsafe {
                                        #crate_path::glib::translate::from_glib(
                                            #crate_path::glib::EnumValue::value(&value)
                                        )
                                    }
                                )
                            },
                            ::std::option::Option::None => {
                                let ty = #crate_path::glib::EnumClass::type_(&self.0);
                                ::std::result::Result::Err(
                                     #crate_path::serde::de::Error::invalid_value(
                                         #crate_path::serde::de::Unexpected::Signed(v as i64),
                                         &self
                                    )
                                )
                            }
                        }
                    }

                    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                    where
                        E: #crate_path::serde::de::Error,
                    {
                        let v = ::std::result::Result::map_err(
                            ::std::convert::TryInto::try_into(v),
                            |_| {
                                #crate_path::serde::de::Error::invalid_value(
                                    #crate_path::serde::de::Unexpected::Signed(v),
                                    &self
                                )
                            }
                        )?;
                        #crate_path::serde::de::Visitor::visit_i32(self, v)
                    }

                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                    where
                        E: #crate_path::serde::de::Error,
                    {
                        let v = ::std::result::Result::map_err(
                            ::std::convert::TryInto::try_into(v),
                            |_| {
                                #crate_path::serde::de::Error::invalid_value(
                                    #crate_path::serde::de::Unexpected::Unsigned(v),
                                    &self
                                )
                            }
                        )?;
                        #crate_path::serde::de::Visitor::visit_i32(self, v)
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: #crate_path::serde::de::Error,
                    {
                        match #crate_path::glib::EnumClass::value_by_nick(&self.0, v) {
                            ::std::option::Option::Some(value) => {
                                ::std::result::Result::Ok(
                                    unsafe {
                                        #crate_path::glib::translate::from_glib(
                                            #crate_path::glib::EnumValue::value(&value)
                                        )
                                    }
                                )
                            },
                            ::std::option::Option::None => {
                                let ty = #crate_path::glib::EnumClass::type_(&self.0);
                                ::std::result::Result::Err(
                                     #crate_path::serde::de::Error::invalid_value(
                                         #crate_path::serde::de::Unexpected::Str(v),
                                         &self
                                    )
                                )
                            }
                        }
                    }
                }

                let ty = <Self as #crate_path::glib::StaticType>::static_type();
                let enum_class = #crate_path::glib::EnumClass::new(ty);

                match enum_class {
                    ::std::option::Option::Some(enum_class) => {
                        deserializer.#deserialize(EnumVisitor(enum_class))
                    },
                    ::std::option::Option::None => {
                        ::std::result::Result::Err(
                            #crate_path::serde::de::Error::custom(
                                format!("No such enum {}", ty)
                            )
                        )
                    }
                }
            }
        }
    }
}

pub fn impl_flags_serialize(input: syn::DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let crate_path = super::crate_path();
    match &input.data {
        syn::Data::Struct(s) => {
            abort!(
                s.struct_token,
                "#[derive(glib_serde::FlagsSerialize)] is not available for structs"
            );
        }
        syn::Data::Union(u) => {
            abort!(
                u.union_token,
                "#[derive(glib_serde::FlagsSerialize)] is not available for unions"
            );
        }
        syn::Data::Enum(_) => (),
    };
    let repr_attr = get_glib_serde_repr_attr(&input);

    let serialize = if repr_attr.is_some() {
        if let Some(attr) = get_repr_attr(&input) {
            abort!(attr, "#[glib_serde_repr] cannot be used with #[repr]");
        }

        quote! {
            serialize_u32(
                <Self as #crate_path::glib::translate::IntoGlib>::into_glib(*self)
            )
        }
    } else {
        quote! {
            serialize_str(&{
                let value: #crate_path::FlagsValue<#ident> = self.into();
                value.to_string()
            })
        }
    };

    quote! {
        impl #crate_path::serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: #crate_path::serde::Serializer
            {
                serializer.#serialize
            }
        }
    }
}

pub fn impl_flags_deserialize(input: syn::DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let crate_path = super::crate_path();
    match &input.data {
        syn::Data::Struct(s) => {
            abort!(
                s.struct_token,
                "#[derive(glib_serde::FlagsDeserialize)] is not available for structs"
            );
        }
        syn::Data::Union(u) => {
            abort!(
                u.union_token,
                "#[derive(glib_serde::FlagsDeserialize)] is not available for unions"
            );
        }
        syn::Data::Enum(_) => (),
    };
    let repr_attr = get_glib_serde_repr_attr(&input);

    let deserialize = if repr_attr.is_some() {
        if let Some(attr) = get_repr_attr(&input) {
            abort!(attr, "#[glib_serde_repr] cannot be used with #[repr]");
        }

        quote! { deserialize_u32 }
    } else {
        quote! { deserialize_str }
    };

    quote! {
        impl<'de> #crate_path::serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: #crate_path::serde::Deserializer<'de>,
            {
                struct FlagsVisitor(#crate_path::glib::FlagsClass);

                impl<'de> #crate_path::serde::de::Visitor<'de> for FlagsVisitor {
                    type Value = #ident;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(
                            formatter,
                            "a valid flags value for {}",
                            &#crate_path::glib::FlagsClass::type_(&self.0)
                        )
                    }

                    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
                    where
                        E: #crate_path::serde::de::Error,
                    {
                        unsafe {
                            ::std::result::Result::Ok(
                                #crate_path::glib::translate::FromGlib::from_glib(v)
                            )
                        }
                    }

                    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                    where
                        E: #crate_path::serde::de::Error,
                    {
                        let v = ::std::result::Result::map_err(
                            ::std::convert::TryInto::try_into(v),
                            |_| {
                                #crate_path::serde::de::Error::invalid_value(
                                    #crate_path::serde::de::Unexpected::Unsigned(v),
                                    &self
                                )
                            }
                        )?;
                        #crate_path::serde::de::Visitor::visit_u32(self, v)
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: #crate_path::serde::de::Error,
                    {
                        ::std::result::Result::map_err(
                            ::std::result::Result::map(
                                str::parse::<#crate_path::FlagsValue<#ident>>(v),
                                |v| #crate_path::FlagsValue::value(&v)
                            ),
                            |_| {
                                let ty = #crate_path::glib::FlagsClass::type_(&self.0);
                                #crate_path::serde::de::Error::invalid_value(
                                    #crate_path::serde::de::Unexpected::Str(v),
                                    &self
                                )
                            }
                        )
                    }
                }

                let ty = <Self as #crate_path::glib::StaticType>::static_type();
                let flags_class = #crate_path::glib::FlagsClass::new(ty);

                match flags_class {
                    ::std::option::Option::Some(flags_class) => {
                        deserializer.#deserialize(FlagsVisitor(flags_class))
                    },
                    ::std::option::Option::None => {
                        ::std::result::Result::Err(
                            #crate_path::serde::de::Error::custom(
                                format!("No such flags {}", ty)
                            )
                        )
                    }
                }
            }
        }
    }
}
