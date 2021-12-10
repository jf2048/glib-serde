use glib::{
    translate::{FromGlib, IntoGlib},
    EnumClass, StaticType,
};
use std::marker::PhantomData;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct EnumValue<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> {
    value: i32,
    phantom: PhantomData<T>,
}

unsafe impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> Send for EnumValue<T> {}
unsafe impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> Sync for EnumValue<T> {}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> EnumValue<T> {
    #[inline]
    pub fn value(&self) -> T {
        unsafe { T::from_glib(self.value) }
    }
    pub fn enum_class() -> EnumClass {
        EnumClass::new(T::static_type())
            .unwrap_or_else(|| panic!("Invalid enum {}", T::static_type()))
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32> + Default> Default for EnumValue<T> {
    fn default() -> Self {
        Self {
            value: <T as Default>::default().into_glib(),
            phantom: Default::default(),
        }
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> From<T> for EnumValue<T> {
    fn from(value: T) -> Self {
        Self {
            value: value.into_glib(),
            phantom: PhantomData,
        }
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32> + Copy> From<&T> for EnumValue<T> {
    fn from(value: &T) -> Self {
        Self {
            value: value.into_glib(),
            phantom: PhantomData,
        }
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> glib::StaticVariantType
    for EnumValue<T>
{
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::Borrowed(glib::VariantTy::STRING)
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> glib::ToVariant for EnumValue<T> {
    fn to_variant(&self) -> glib::Variant {
        let class = Self::enum_class();
        let value = class.value(self.value).unwrap_or_else(|| {
            panic!(
                "Invalid value '{}' for enum {}",
                self.value,
                T::static_type()
            )
        });
        value.nick().to_variant()
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> glib::FromVariant for EnumValue<T> {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        let class = Self::enum_class();
        variant
            .str()
            .and_then(|s| class.value_by_nick(s))
            .map(|v| Self {
                value: v.value(),
                phantom: PhantomData,
            })
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> super::VariantType for EnumValue<T> {}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> std::fmt::Display for EnumValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let class = Self::enum_class();
        let value = class.value(self.value).ok_or(std::fmt::Error)?;
        value.nick().fmt(f)
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> std::str::FromStr for EnumValue<T> {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let class = Self::enum_class();
        let value = class.value_by_nick(s).ok_or_else(|| ParseEnumError {
            token: s.to_owned(),
        })?;
        Ok(Self {
            value: value.value(),
            phantom: PhantomData,
        })
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> serde::ser::Serialize
    for EnumValue<T>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let class = Self::enum_class();
        let value = class.value(self.value).ok_or_else(|| {
            serde::ser::Error::custom(format!(
                "Invalid value '{}' for enum {}",
                self.value,
                T::static_type()
            ))
        })?;
        serializer.serialize_str(value.nick())
    }
}

impl<'de, T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> serde::de::Deserialize<'de>
    for EnumValue<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct EnumVisitor(EnumClass);

        impl<'de> serde::de::Visitor<'de> for EnumVisitor {
            type Value = i32;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a valid string for enum {}", self.0.type_())
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value = self.0.value_by_nick(v).ok_or_else(|| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &format!("valid string for enum {}", self.0.type_()).as_str(),
                    )
                })?;
                Ok(value.value())
            }
        }

        let class = Self::enum_class();
        let value = deserializer.deserialize_str(EnumVisitor(class))?;
        Ok(EnumValue {
            value,
            phantom: PhantomData,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct EnumReprValue<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> {
    value: i32,
    phantom: PhantomData<T>,
}

unsafe impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> Send for EnumReprValue<T> {}
unsafe impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> Sync for EnumReprValue<T> {}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> EnumReprValue<T> {
    #[inline]
    pub fn value(&self) -> T {
        unsafe { T::from_glib(self.value) }
    }
    pub fn enum_class() -> EnumClass {
        EnumClass::new(T::static_type())
            .unwrap_or_else(|| panic!("Invalid enum {}", T::static_type()))
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32> + Default> Default
    for EnumReprValue<T>
{
    fn default() -> Self {
        Self {
            value: <T as Default>::default().into_glib(),
            phantom: Default::default(),
        }
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> From<T> for EnumReprValue<T> {
    fn from(value: T) -> Self {
        Self {
            value: value.into_glib(),
            phantom: PhantomData,
        }
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32> + Copy> From<&T>
    for EnumReprValue<T>
{
    fn from(value: &T) -> Self {
        Self {
            value: value.into_glib(),
            phantom: PhantomData,
        }
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> glib::StaticVariantType
    for EnumReprValue<T>
{
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::Borrowed(glib::VariantTy::INT32)
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> glib::ToVariant
    for EnumReprValue<T>
{
    fn to_variant(&self) -> glib::Variant {
        let class = Self::enum_class();
        let value = class.value(self.value).unwrap_or_else(|| {
            panic!(
                "Invalid value '{}' for enum {}",
                self.value,
                T::static_type()
            )
        });
        value.value().to_variant()
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> glib::FromVariant
    for EnumReprValue<T>
{
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        let class = Self::enum_class();
        variant
            .get::<i32>()
            .and_then(|s| class.value(s))
            .map(|v| Self {
                value: v.value(),
                phantom: PhantomData,
            })
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> super::VariantType
    for EnumReprValue<T>
{
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> std::fmt::Display
    for EnumReprValue<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let class = Self::enum_class();
        let value = class.value(self.value).ok_or(std::fmt::Error)?;
        value.nick().fmt(f)
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> std::str::FromStr
    for EnumReprValue<T>
{
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let class = Self::enum_class();
        let value = class.value_by_nick(s).ok_or_else(|| ParseEnumError {
            token: s.to_owned(),
        })?;
        Ok(Self {
            value: value.value(),
            phantom: PhantomData,
        })
    }
}

impl<T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> serde::ser::Serialize
    for EnumReprValue<T>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let class = Self::enum_class();
        let value = class.value(self.value).ok_or_else(|| {
            serde::ser::Error::custom(format!(
                "Invalid value '{}' for enum {}",
                self.value,
                T::static_type()
            ))
        })?;
        serializer.serialize_i32(value.value())
    }
}

impl<'de, T: StaticType + FromGlib<i32> + IntoGlib<GlibType = i32>> serde::de::Deserialize<'de>
    for EnumReprValue<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct EnumVisitor(EnumClass);

        impl<'de> serde::de::Visitor<'de> for EnumVisitor {
            type Value = i32;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a valid integer for enum {}", self.0.type_())
            }
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value = self.0.value(v).ok_or_else(|| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Signed(v as i64),
                        &format!("valid integer for enum {}", self.0.type_()).as_str(),
                    )
                })?;
                Ok(value.value())
            }
        }

        let class = Self::enum_class();
        let value = deserializer.deserialize_i32(EnumVisitor(class))?;
        Ok(EnumReprValue {
            value,
            phantom: PhantomData,
        })
    }
}

pub struct ParseEnumError {
    token: String,
}

impl std::fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid enum value `{}`", self.token)
    }
}
