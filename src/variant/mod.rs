use crate::{VariantBuilder, VariantBuilderExt, VariantType};
use glib::{translate::*, variant::VariantTypeMismatchError, VariantTy};
use std::{borrow::Cow, ops::Deref};

pub(crate) mod deserialize;
pub(crate) mod deserializer;
pub use deserializer::*;
pub(crate) mod serialize;
pub(crate) mod serializer;
pub use serializer::*;

const STRUCT_NAME: &str = "glib_serde::$Variant";

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Variant(glib::Variant);

unsafe impl Send for Variant {}
unsafe impl Sync for Variant {}

impl std::fmt::Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl glib::StaticVariantType for Variant {
    fn static_variant_type() -> Cow<'static, VariantTy> {
        <glib::Variant as glib::StaticVariantType>::static_variant_type()
    }
}

impl Deref for Variant {
    type Target = glib::Variant;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl VariantType for Variant {}

impl From<glib::Variant> for Variant {
    fn from(other: glib::Variant) -> Self {
        Self(other)
    }
}

impl From<Variant> for glib::Variant {
    fn from(other: Variant) -> Self {
        other.0
    }
}

pub trait GlibVariantExt {
    fn from_none(type_: &VariantTy) -> glib::Variant;
    fn from_some(value: &glib::Variant) -> glib::Variant;
    fn from_dict_entry(key: &glib::Variant, value: &glib::Variant) -> glib::Variant;
    fn array_from_variant_iter(
        ty: &VariantTy,
        children: impl IntoIterator<Item = glib::Variant>,
    ) -> glib::Variant;
    fn is_of_type(&self, ty: &VariantTy) -> Result<(), VariantTypeMismatchError>;
    fn maybe(&self) -> Option<Option<glib::Variant>>;
}

impl GlibVariantExt for glib::Variant {
    fn from_none(type_: &VariantTy) -> glib::Variant {
        unsafe {
            from_glib_none(glib::ffi::g_variant_new_maybe(
                type_.as_ptr() as *const _,
                std::ptr::null_mut(),
            ))
        }
    }
    fn from_some(value: &glib::Variant) -> glib::Variant {
        unsafe {
            from_glib_none(glib::ffi::g_variant_new_maybe(
                std::ptr::null(),
                value.to_glib_none().0 as *mut glib::ffi::GVariant,
            ))
        }
    }
    fn from_dict_entry(key: &glib::Variant, value: &glib::Variant) -> glib::Variant {
        unsafe {
            from_glib_none(glib::ffi::g_variant_new_dict_entry(
                key.to_glib_none().0,
                value.to_glib_none().0,
            ))
        }
    }
    fn array_from_variant_iter(
        ty: &VariantTy,
        children: impl IntoIterator<Item = glib::Variant>,
    ) -> glib::Variant {
        assert!(ty.is_array());
        let builder = VariantBuilder::new(ty);
        for value in children {
            assert!(
                value.is_of_type(ty.element()).is_ok(),
                "Type mismatch: Expected `{}` got `{}`",
                ty.element(),
                value.type_(),
            );
            unsafe {
                builder.add_value(&value);
            }
        }
        builder.end()
    }
    fn is_of_type(&self, ty: &VariantTy) -> Result<(), VariantTypeMismatchError> {
        let is_type: bool = unsafe {
            from_glib(glib::ffi::g_variant_is_of_type(
                self.to_glib_none().0,
                ty.to_glib_none().0,
            ))
        };
        if is_type {
            Ok(())
        } else {
            Err(VariantTypeMismatchError::new(
                self.type_().to_owned(),
                ty.to_owned(),
            ))
        }
    }
    fn maybe(&self) -> Option<Option<glib::Variant>> {
        self.is_of_type(VariantTy::MAYBE).ok()?;
        Some(unsafe {
            let child = glib::ffi::g_variant_get_maybe(self.to_glib_none().0);
            if child.is_null() {
                None
            } else {
                Some(from_glib_full(child))
            }
        })
    }
}

pub trait VariantSerializeExt {
    fn as_serializable(&self) -> &Variant;
}

impl VariantSerializeExt for glib::Variant {
    fn as_serializable(&self) -> &Variant {
        unsafe { &*(self as *const glib::Variant as *const Variant) }
    }
}
