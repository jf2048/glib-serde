// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

use crate::VariantType;

pub(crate) const STRUCT_NAME: &str = "glib_serde::$Signature";

/// Wrapper type for [`Variant`](struct@glib::Variant)s of type
/// [`SIGNATURE`](glib::VariantTy::SIGNATURE).
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature(glib::VariantType);

unsafe impl Send for Signature {}
unsafe impl Sync for Signature {}

impl Signature {
    pub fn new(s: impl Into<glib::GString>) -> Result<Self, glib::BoolError> {
        let s = s.into();
        let valid = unsafe { glib::ffi::g_variant_is_signature(s.as_ptr() as *const _) };
        if valid == glib::ffi::GFALSE {
            Err(glib::bool_error!("Invalid signature: {}", s))
        } else {
            Ok(Self(glib::VariantType::from_string(s)?))
        }
    }
}

impl std::ops::Deref for Signature {
    type Target = glib::VariantTy;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<glib::VariantType> for Signature {
    type Error = glib::BoolError;

    fn try_from(value: glib::VariantType) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl TryFrom<&glib::VariantTy> for Signature {
    type Error = glib::BoolError;

    fn try_from(value: &glib::VariantTy) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl From<Signature> for glib::VariantType {
    fn from(sig: Signature) -> Self {
        sig.0
    }
}

impl glib::StaticVariantType for Signature {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::Borrowed(glib::VariantTy::SIGNATURE)
    }
}

impl glib::ToVariant for Signature {
    fn to_variant(&self) -> glib::Variant {
        unsafe {
            glib::translate::from_glib_none(glib::ffi::g_variant_new_signature(
                self.0.as_ptr() as *const _
            ))
        }
    }
}

impl glib::FromVariant for Signature {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        variant.str().and_then(|s| Self::new(s).ok())
    }
}

impl VariantType for Signature {}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for Signature {
    type Err = glib::BoolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl serde::ser::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_newtype_struct(STRUCT_NAME, self.0.as_str())
    }
}

impl<'de> serde::de::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct StrVisitor;

        impl<'de> serde::de::Visitor<'de> for StrVisitor {
            type Value = Signature;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid D-Bus signature")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = <String as serde::Deserialize>::deserialize(deserializer)?;
                Signature::new(s.clone()).map_err(|_| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &self)
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let s = seq.next_element::<String>()?.ok_or_else(|| {
                    serde::de::Error::invalid_length(0, &"tuple struct Signature with 1 element")
                })?;
                Signature::new(s.clone()).map_err(|_| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &self)
                })
            }
        }

        deserializer.deserialize_newtype_struct(STRUCT_NAME, StrVisitor)
    }
}
