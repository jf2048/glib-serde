use glib::{
    translate::{FromGlib, IntoGlib},
    FlagsClass, StaticType,
};
use std::marker::PhantomData;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct FlagsValue<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> {
    value: u32,
    phantom: PhantomData<T>,
}

unsafe impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> Send for FlagsValue<T> {}
unsafe impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> Sync for FlagsValue<T> {}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> FlagsValue<T> {
    #[inline]
    pub fn value(&self) -> T {
        unsafe { T::from_glib(self.value) }
    }
    pub fn flags_class() -> FlagsClass {
        FlagsClass::new(T::static_type())
            .unwrap_or_else(|| panic!("Invalid flags {}", T::static_type()))
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32> + Default> Default for FlagsValue<T> {
    fn default() -> Self {
        Self {
            value: <T as Default>::default().into_glib(),
            phantom: Default::default(),
        }
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> From<T> for FlagsValue<T> {
    fn from(value: T) -> Self {
        Self {
            value: value.into_glib(),
            phantom: PhantomData,
        }
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32> + Copy> From<&T> for FlagsValue<T> {
    fn from(value: &T) -> Self {
        Self {
            value: value.into_glib(),
            phantom: PhantomData,
        }
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> glib::StaticVariantType
    for FlagsValue<T>
{
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::Borrowed(glib::VariantTy::STRING)
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> glib::ToVariant for FlagsValue<T> {
    fn to_variant(&self) -> glib::Variant {
        self.to_string().to_variant()
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> glib::FromVariant for FlagsValue<T> {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        variant.str().and_then(|s| s.parse().ok())
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> super::VariantType
    for FlagsValue<T>
{
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> std::fmt::Display for FlagsValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let class = Self::flags_class();
        let mut s = String::new();
        let mut value = self.value;
        for val in class.values() {
            let v = val.value();
            if (value & v) == v {
                value &= !v;
                if !s.is_empty() {
                    s.push('|');
                }
                s.push_str(val.nick());
            }
        }
        s.fmt(f)
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> std::str::FromStr for FlagsValue<T> {
    type Err = ParseFlagsError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self {
                value: 0,
                phantom: PhantomData,
            });
        }
        let class = Self::flags_class();
        let mut count = 0usize;
        let mut cur_start = s;
        let mut value = 0u32;
        loop {
            let is_end = s.is_empty();
            if is_end || &s[0..1] == "|" {
                let item = &cur_start[0..count];
                let v = class.value_by_nick(item).ok_or_else(|| ParseFlagsError {
                    token: item.to_owned(),
                })?;
                value |= v.value();
                if is_end {
                    break;
                }
                count = 0;
                s = &s[1..];
                cur_start = s;
            } else {
                s = &s[1..];
                count += 1;
            }
        }
        Ok(Self {
            value,
            phantom: PhantomData,
        })
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> serde::ser::Serialize
    for FlagsValue<T>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> serde::de::Deserialize<'de>
    for FlagsValue<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct FlagsVisitor<T>(FlagsClass, PhantomData<T>);

        impl<'de, T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> serde::de::Visitor<'de>
            for FlagsVisitor<T>
        {
            type Value = FlagsValue<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a valid string for flags {}", self.0.type_())
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse::<FlagsValue<T>>().map_err(|_| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &format!("valid string for flags {}", self.0.type_()).as_str(),
                    )
                })
            }
        }

        let class = Self::flags_class();
        deserializer.deserialize_str(FlagsVisitor::<T>(class, PhantomData))
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub struct FlagsReprValue<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> {
    value: u32,
    phantom: PhantomData<T>,
}

unsafe impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> Send for FlagsReprValue<T> {}
unsafe impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> Sync for FlagsReprValue<T> {}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> FlagsReprValue<T> {
    #[inline]
    pub fn value(&self) -> T {
        unsafe { T::from_glib(self.value) }
    }
    pub fn flags_class() -> FlagsClass {
        FlagsClass::new(T::static_type())
            .unwrap_or_else(|| panic!("Invalid flags {}", T::static_type()))
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32> + Default> Default
    for FlagsReprValue<T>
{
    fn default() -> Self {
        Self {
            value: <T as Default>::default().into_glib(),
            phantom: Default::default(),
        }
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> From<T> for FlagsReprValue<T> {
    fn from(value: T) -> Self {
        Self {
            value: value.into_glib(),
            phantom: PhantomData,
        }
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32> + Copy> From<&T>
    for FlagsReprValue<T>
{
    fn from(value: &T) -> Self {
        Self {
            value: value.into_glib(),
            phantom: PhantomData,
        }
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> glib::StaticVariantType
    for FlagsReprValue<T>
{
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::Borrowed(glib::VariantTy::UINT32)
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> glib::ToVariant
    for FlagsReprValue<T>
{
    fn to_variant(&self) -> glib::Variant {
        self.value.to_variant()
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> glib::FromVariant
    for FlagsReprValue<T>
{
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        variant.get::<u32>().map(|v| Self {
            value: v,
            phantom: PhantomData,
        })
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> super::VariantType
    for FlagsReprValue<T>
{
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> std::fmt::Display
    for FlagsReprValue<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let class = Self::flags_class();
        let mut s = String::new();
        let mut value = self.value;
        for val in class.values() {
            let v = val.value();
            if (value & v) == v {
                value &= !v;
                if !s.is_empty() {
                    s.push('|');
                }
                s.push_str(val.nick());
            }
        }
        s.fmt(f)
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> std::str::FromStr
    for FlagsReprValue<T>
{
    type Err = ParseFlagsError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self {
                value: 0,
                phantom: PhantomData,
            });
        }
        let class = Self::flags_class();
        let mut count = 0usize;
        let mut cur_start = s;
        let mut value = 0u32;
        loop {
            let is_end = s.is_empty();
            if is_end || &s[0..1] == "|" {
                let item = &cur_start[0..count];
                let v = class.value_by_nick(item).ok_or_else(|| ParseFlagsError {
                    token: item.to_owned(),
                })?;
                value |= v.value();
                if is_end {
                    break;
                }
                count = 0;
                s = &s[1..];
                cur_start = s;
            } else {
                s = &s[1..];
                count += 1;
            }
        }
        Ok(Self {
            value,
            phantom: PhantomData,
        })
    }
}

impl<T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> serde::ser::Serialize
    for FlagsReprValue<T>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_u32(self.value)
    }
}

impl<'de, T: StaticType + FromGlib<u32> + IntoGlib<GlibType = u32>> serde::de::Deserialize<'de>
    for FlagsReprValue<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct FlagsVisitor(FlagsClass);

        impl<'de> serde::de::Visitor<'de> for FlagsVisitor {
            type Value = u32;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a valid unsigned integer for flags {}",
                    self.0.type_()
                )
            }
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v)
            }
        }

        let class = Self::flags_class();
        let value = deserializer.deserialize_u32(FlagsVisitor(class))?;
        Ok(FlagsReprValue {
            value,
            phantom: PhantomData,
        })
    }
}

pub struct ParseFlagsError {
    token: String,
}

impl std::fmt::Display for ParseFlagsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid flag value `{}`", self.token)
    }
}
