// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

use super::{GlibVariantExt, Variant};
use crate::{
    object_path, signature, Error, ObjectPath, Signature, VariantBuilder, VariantBuilderExt,
};
use glib::{ToVariant, VariantTy};
use serde::{
    de::{self, DeserializeSeed, Visitor},
    Deserialize, Deserializer,
};

impl<'de> Deserialize<'de> for Variant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_tuple_struct(super::STRUCT_NAME, 2, VariantVisitor)
            .map(Into::into)
    }
}

struct VariantVisitor;

impl<'de> Visitor<'de> for VariantVisitor {
    type Value = glib::Variant;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("GVariant tuple of length 2")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let tag = seq
            .next_element::<String>()?
            .ok_or_else(|| de::Error::invalid_length(0, &"tuple struct Variant with 2 elements"))?;
        let ty = VariantTy::new(&tag).map_err(de::Error::custom)?;
        if !ty.is_definite() {
            return Err(de::Error::custom("Type must be definite"));
        }
        let seed = VariantDeserializeInput(ty);
        let value = seq
            .next_element_seed(seed)?
            .ok_or_else(|| de::Error::invalid_length(1, &"tuple struct Variant with 2 elements"))?;
        Ok(value)
    }
}

#[repr(transparent)]
struct VariantDeserializeInput<'t>(&'t VariantTy);

impl<'t, 'de> DeserializeSeed<'de> for VariantDeserializeInput<'t> {
    type Value = glib::Variant;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ty = self.0;
        let visitor = VariantValueVisitor(ty);
        if ty.is_basic() {
            match ty.as_str() {
                "b" => deserializer.deserialize_bool(visitor),
                "y" => deserializer.deserialize_u8(visitor),
                "n" => deserializer.deserialize_i16(visitor),
                "q" => deserializer.deserialize_u16(visitor),
                "i" => deserializer.deserialize_i32(visitor),
                "u" => deserializer.deserialize_u32(visitor),
                "x" => deserializer.deserialize_i64(visitor),
                "t" => deserializer.deserialize_u64(visitor),
                "d" => deserializer.deserialize_f64(visitor),
                "s" => deserializer.deserialize_str(visitor),
                "o" => deserializer.deserialize_newtype_struct(object_path::STRUCT_NAME, visitor),
                "g" => deserializer.deserialize_newtype_struct(signature::STRUCT_NAME, visitor),
                "h" => Err(de::Error::custom("HANDLE values not supported")),
                _ => unimplemented!(),
            }
        } else if ty.is_array() {
            let elem = ty.element();
            if elem == VariantTy::BYTE {
                deserializer.deserialize_bytes(visitor)
            } else if ty.element().is_dict_entry() {
                deserializer.deserialize_map(visitor)
            } else {
                deserializer.deserialize_seq(visitor)
            }
        } else if ty.is_tuple() {
            let len = ty.n_items();
            if len > 0 {
                deserializer.deserialize_tuple(len, visitor)
            } else {
                deserializer.deserialize_unit(visitor)
            }
        } else if ty.is_maybe() {
            deserializer.deserialize_option(visitor)
        } else if ty.is_variant() {
            Variant::deserialize(deserializer).map(|v| v.to_variant())
        } else {
            Err(de::Error::custom(Error::UnsupportedType(ty.to_owned())))
        }
    }
}

#[repr(transparent)]
struct VariantValueVisitor<'t>(&'t VariantTy);

impl<'t, 'de> Visitor<'de> for VariantValueVisitor<'t> {
    type Value = glib::Variant;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any valid GVariant value")
    }

    #[inline]
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(value.to_variant())
    }

    #[inline]
    fn visit_i16<E: de::Error>(self, v: i16) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    #[inline]
    fn visit_i32<E: de::Error>(self, v: i32) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    #[inline]
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    #[inline]
    fn visit_u8<E: de::Error>(self, v: u8) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    #[inline]
    fn visit_u16<E: de::Error>(self, v: u16) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    #[inline]
    fn visit_u32<E: de::Error>(self, v: u32) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    #[inline]
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    #[inline]
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        match self.0.as_str() {
            "s" => Ok(v.to_variant()),
            "o" => ObjectPath::new(v)
                .map(|o| o.to_variant())
                .map_err(de::Error::custom),
            "g" => Signature::new(v)
                .map(|g| g.to_variant())
                .map_err(de::Error::custom),
            _ => Err(de::Error::custom(Error::UnsupportedType(self.0.to_owned()))),
        }
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        match self.0.as_str() {
            "s" => Ok(v.to_variant()),
            _ => self.visit_str(&v),
        }
    }

    #[inline]
    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(glib::Variant::array_from_fixed_array(v))
    }

    #[inline]
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(glib::Variant::from_none(self.0.element()))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seed = VariantDeserializeInput(self.0.element());
        let value = seed.deserialize(deserializer)?;
        Ok(glib::Variant::from_some(&value))
    }

    #[inline]
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(().to_variant())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(self)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let ty = self.0;
        if ty.is_array() {
            let builder = VariantBuilder::new(ty);
            let elem = self.0.element();
            while let Some(value) = {
                let seed = VariantDeserializeInput(elem);
                seq.next_element_seed(seed)?
            } {
                value.is_of_type(elem).map_err(de::Error::custom)?;
                unsafe {
                    builder.add_value(&value);
                }
            }
            Ok(builder.end())
        } else if ty.is_tuple() || ty.is_dict_entry() {
            let builder = VariantBuilder::new(ty);
            let len = ty.n_items();
            let mut iter = ty.first();
            for i in 0..len {
                let elem = iter.unwrap();
                let seed = VariantDeserializeInput(elem);
                let value = seq.next_element_seed(seed)?.ok_or_else(|| {
                    de::Error::invalid_length(i, &format!("tuple of length {}", len).as_str())
                })?;
                value.is_of_type(elem).map_err(de::Error::custom)?;
                unsafe {
                    builder.add_value(&value);
                }
                iter = elem.next();
            }
            Ok(builder.end())
        } else if ty.is_definite() {
            let seed = VariantDeserializeInput(ty);
            seq.next_element_seed(seed)?
                .ok_or_else(|| de::Error::invalid_length(0, &"tuple of length 1"))
        } else {
            Err(de::Error::custom(Error::UnsupportedType(ty.to_owned())))
        }
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        if !self.0.is_array() || !self.0.element().is_dict_entry() {
            return Err(de::Error::custom(Error::UnsupportedType(self.0.to_owned())));
        }
        let builder = VariantBuilder::new(self.0);
        let elem = self.0.element();
        let key_type = elem.key();
        let value_type = elem.value();
        while let Some((key, value)) = {
            let kseed = VariantDeserializeInput(key_type);
            let vseed = VariantDeserializeInput(value_type);
            map.next_entry_seed(kseed, vseed)?
        } {
            let dict_entry = builder.open(self.0.element());
            key.is_of_type(key_type).map_err(de::Error::custom)?;
            value.is_of_type(value_type).map_err(de::Error::custom)?;
            unsafe {
                dict_entry.add_value(&key);
                dict_entry.add_value(&value);
            }
        }
        Ok(builder.end())
    }
}
