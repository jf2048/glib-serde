use crate::VariantType;

/// Wrapper object for [`Variant`](struct@glib::Variant)s of type
/// [`OBJECT_PATH`](glib::VariantTy::OBJECT_PATH).
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectPath(glib::GString);

unsafe impl Send for ObjectPath {}
unsafe impl Sync for ObjectPath {}

impl ObjectPath {
    pub fn new(s: impl Into<glib::GString>) -> Result<Self, glib::BoolError> {
        let s = s.into();
        let valid = unsafe { glib::ffi::g_variant_is_object_path(s.as_ptr() as *const _) };
        if valid == glib::ffi::GFALSE {
            Err(glib::bool_error!("Invalid object path: {}", s))
        } else {
            Ok(Self(s))
        }
    }
    pub unsafe fn new_unchecked(s: impl Into<glib::GString>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl glib::StaticVariantType for ObjectPath {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::Borrowed(glib::VariantTy::OBJECT_PATH)
    }
}

impl glib::ToVariant for ObjectPath {
    fn to_variant(&self) -> glib::Variant {
        unsafe { glib::translate::from_glib_none(glib::ffi::g_variant_new_object_path(self.0.as_ptr() as *const _)) }
    }
}

impl glib::FromVariant for ObjectPath {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        variant.str().and_then(|s| Self::new(s).ok())
    }
}

impl VariantType for ObjectPath {}

impl std::fmt::Display for ObjectPath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl serde::ser::Serialize for ObjectPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> serde::de::Deserialize<'de> for ObjectPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct StrVisitor;

        impl<'de> serde::de::Visitor<'de> for StrVisitor {
            type Value = ObjectPath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid D-Bus object path")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error
            {
                use serde::de::Error;
                Ok(ObjectPath::new(v).map_err(|e| Error::custom(e.to_string()))?)
            }
        }

        deserializer.deserialize_str(StrVisitor)
    }
}
