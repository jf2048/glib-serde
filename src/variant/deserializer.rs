use super::{GlibVariantExt, Variant};
use crate::Error;
use glib::{FixedSizeVariantType, VariantClass, VariantTy};
use serde::{
    de::{self, IntoDeserializer, Visitor},
    Deserialize, Deserializer,
};

/// Deserializes `T` from a [`glib::Variant`](struct@glib::Variant).
pub fn from_variant<'de, T>(variant: &'de glib::Variant) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    T::deserialize(variant.as_serializable())
}

impl<'de> de::Deserializer<'de> for &Variant {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.classify() {
            VariantClass::Boolean => self.deserialize_bool(visitor),
            VariantClass::Byte => self.deserialize_u8(visitor),
            VariantClass::Int16 => self.deserialize_i16(visitor),
            VariantClass::Uint16 => self.deserialize_u16(visitor),
            VariantClass::Int32 => self.deserialize_i32(visitor),
            VariantClass::Uint32 => self.deserialize_u32(visitor),
            VariantClass::Int64 => self.deserialize_i64(visitor),
            VariantClass::Uint64 => self.deserialize_u64(visitor),
            VariantClass::Double => self.deserialize_bool(visitor),
            VariantClass::String | VariantClass::ObjectPath | VariantClass::Signature => {
                self.deserialize_str(visitor)
            }
            VariantClass::Variant => {
                let variant = self.try_get::<glib::Variant>()?;
                variant.as_serializable().deserialize_any(visitor)
            }
            VariantClass::Maybe => self.deserialize_option(visitor),
            VariantClass::Array => {
                let elem = self.type_().element();
                if elem == VariantTy::BYTE {
                    self.deserialize_bytes(visitor)
                } else if elem.is_dict_entry() {
                    self.deserialize_map(visitor)
                } else {
                    self.deserialize_seq(visitor)
                }
            }
            VariantClass::Tuple => {
                let len = self.n_children();
                if len > 0 {
                    self.deserialize_tuple(len, visitor)
                } else {
                    self.deserialize_unit(visitor)
                }
            }
            VariantClass::DictEntry => self.deserialize_tuple(2, visitor),
            _ => Err(Error::UnsupportedType(self.type_().to_owned())),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bool(self.try_get()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i8(self.try_get::<i16>()?.try_into()?)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i16(self.try_get()?)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i32(self.try_get()?)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_i64(self.try_get()?)
    }

    serde::serde_if_integer128! {
        fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            let buf = self.fixed_array::<i64>()?;
            if buf.len() != 2 {
                return Err(Error::LengthMismatch { actual: buf.len(), expected: 2 });
            }
            visitor.visit_i128(((buf[0] as u128) << 64) as i128 + buf[1] as i128)
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u8(self.try_get()?)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u16(self.try_get()?)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u32(self.try_get()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u64(self.try_get()?)
    }

    serde::serde_if_integer128! {
        fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>
        {
            let buf = self.fixed_array::<u64>()?;
            if buf.len() != 2 {
                return Err(Error::LengthMismatch { actual: buf.len(), expected: 2 });
            }
            visitor.visit_u128(((buf[0] as u128) << 64) + buf[1] as u128)
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f32(self.try_get::<f64>()? as f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f64(self.try_get()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self
            .str()
            .ok_or_else(|| Error::StrMismatch(self.type_().to_owned()))?;
        let c = s
            .chars()
            .next()
            .ok_or_else(|| Error::ExpectedChar(s.to_owned()))?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self
            .str()
            .ok_or_else(|| Error::StrMismatch(self.type_().to_owned()))?;
        visitor.visit_str(s)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self
            .str()
            .ok_or_else(|| Error::StrMismatch(self.type_().to_owned()))?;
        visitor.visit_string(s.to_owned())
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bytes(self.fixed_array()?)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_byte_buf(self.fixed_array()?.to_owned())
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.is_of_type(VariantTy::MAYBE)?;
        match self.maybe().unwrap() {
            Some(child) => visitor.visit_some(child.as_serializable()),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.try_get::<()>()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let ty = self.type_();
        if ty.is_array() {
            match ty.element().as_str() {
                "y" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, u8>::new(self.fixed_array()?)),
                "q" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, u16>::new(
                    self.fixed_array()?,
                )),
                "u" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, u32>::new(
                    self.fixed_array()?,
                )),
                "t" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, u64>::new(
                    self.fixed_array()?,
                )),
                "n" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, i16>::new(
                    self.fixed_array()?,
                )),
                "i" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, i32>::new(
                    self.fixed_array()?,
                )),
                "x" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, i64>::new(
                    self.fixed_array()?,
                )),
                "d" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, f64>::new(
                    self.fixed_array()?,
                )),
                "b" => visitor.visit_seq(FixedSeqDeserializer::<'_, '_, bool>::new(
                    self.fixed_array()?,
                )),
                _ => visitor.visit_seq(ContainerDeserializer::new(self)),
            }
        } else if ty.is_tuple() {
            visitor.visit_seq(ContainerDeserializer::new(self))
        } else {
            Err(Error::UnsupportedType(ty.to_owned()))
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.is_of_type(VariantTy::TUPLE)?;
        if self.n_children() != len {
            return Err(Error::LengthMismatch {
                actual: self.n_children(),
                expected: len,
            });
        }
        visitor.visit_seq(ContainerDeserializer::new(self))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if name == super::STRUCT_NAME {
            self.is_of_type(VariantTy::VARIANT)?;
            assert_eq!(len, 2);
            let inner = self.as_variant().unwrap();
            visitor.visit_seq(VariantDeserializer::new(inner.as_serializable()))
        } else {
            self.deserialize_tuple(len, visitor)
        }
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.is_of_type(VariantTy::DICTIONARY)?;
        visitor.visit_map(ContainerDeserializer::new(self))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if self.is_container() {
            visitor.visit_enum(EnumDeserializer::new(self))
        } else {
            visitor.visit_enum(UnitEnumDeserializer::new(self))
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.classify() {
            VariantClass::Byte => self.deserialize_u8(visitor),
            VariantClass::Int16 => self.deserialize_i16(visitor),
            VariantClass::Uint16 => self.deserialize_u16(visitor),
            VariantClass::Int32 => self.deserialize_i32(visitor),
            VariantClass::Uint32 => self.deserialize_u32(visitor),
            VariantClass::Int64 => self.deserialize_i64(visitor),
            VariantClass::Uint64 => self.deserialize_u64(visitor),
            VariantClass::String => self.deserialize_str(visitor),
            _ => Err(Error::UnsupportedType(self.type_().to_owned())),
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_any(visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

#[repr(transparent)]
struct EnumDeserializer<'v> {
    input: &'v Variant,
}

impl<'v> EnumDeserializer<'v> {
    fn new(input: &'v Variant) -> Self {
        Self { input }
    }
    fn value(&self) -> Result<Variant, Error> {
        self.input
            .try_child_value(1)
            .and_then(|v| v.as_variant())
            .ok_or_else(|| Error::UnsupportedType(self.input.type_().to_owned()))
            .map(Into::into)
    }
}

impl<'v, 'de> de::EnumAccess<'de> for EnumDeserializer<'v> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let tag = self
            .input
            .try_child_value(0)
            .ok_or_else(|| Error::UnsupportedType(self.input.type_().to_owned()))?;
        let value = seed.deserialize(tag.as_serializable())?;
        Ok((value, self))
    }
}

impl<'v, 'de> de::VariantAccess<'de> for EnumDeserializer<'v> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        self.value()?.is_of_type(VariantTy::UNIT)?;
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&self.value()?)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value()?.deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value()?.deserialize_seq(visitor)
    }
}

#[repr(transparent)]
struct UnitEnumDeserializer<'v> {
    input: &'v Variant,
}

impl<'v> UnitEnumDeserializer<'v> {
    fn new(input: &'v Variant) -> Self {
        Self { input }
    }
}

impl<'v, 'de> de::EnumAccess<'de> for UnitEnumDeserializer<'v> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self.input)?;
        Ok((value, self))
    }
}

impl<'v, 'de> de::VariantAccess<'de> for UnitEnumDeserializer<'v> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::UnsupportedType(self.input.type_().to_owned()))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType(self.input.type_().to_owned()))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType(self.input.type_().to_owned()))
    }
}

struct FixedSeqDeserializer<'v, 'de, V: FixedSizeVariantType + IntoDeserializer<'de, Error>> {
    input: &'v [V],
    index: usize,
    phantom: std::marker::PhantomData<&'de ()>,
}

impl<'v, 'de, V: FixedSizeVariantType + IntoDeserializer<'de, Error>>
    FixedSeqDeserializer<'v, 'de, V>
{
    fn new(input: &'v [V]) -> Self {
        Self {
            input,
            index: 0,
            phantom: Default::default(),
        }
    }
}

impl<'v, 'de, V: FixedSizeVariantType + IntoDeserializer<'de, Error>> de::SeqAccess<'de>
    for FixedSeqDeserializer<'v, 'de, V>
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index >= self.input.len() {
            return Ok(None);
        }
        let child = self.input[self.index];
        self.index += 1;
        seed.deserialize(child.into_deserializer()).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.input.len() - self.index - 1)
    }
}

struct VariantDeserializer<'v> {
    input: &'v Variant,
    index: usize,
}

impl<'v> VariantDeserializer<'v> {
    fn new(input: &'v Variant) -> Self {
        Self { input, index: 0 }
    }
}

impl<'v, 'de> de::SeqAccess<'de> for VariantDeserializer<'v> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.index {
            0 => {
                self.index += 1;
                let deserializer = self.input.type_().as_str().into_deserializer();
                seed.deserialize(deserializer).map(Some)
            },
            1 => {
                self.index += 1;
                seed.deserialize(self.input).map(Some)
            },
            _ => Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.input.n_children() - self.index - 1)
    }
}

struct ContainerDeserializer<'v> {
    input: &'v Variant,
    index: usize,
}

impl<'v> ContainerDeserializer<'v> {
    fn new(input: &'v Variant) -> Self {
        Self { input, index: 0 }
    }
}

impl<'v, 'de> de::SeqAccess<'de> for ContainerDeserializer<'v> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index >= self.input.n_children() {
            return Ok(None);
        }
        let child = self.input.child_value(self.index);
        self.index += 1;
        seed.deserialize(child.as_serializable()).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.input.n_children() - self.index - 1)
    }
}

impl<'v, 'de> de::MapAccess<'de> for ContainerDeserializer<'v> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.index >= self.input.n_children() {
            return Ok(None);
        }
        let entry = self.input.child_value(self.index);
        let key = entry.child_value(0);
        seed.deserialize(key.as_serializable()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let entry = self.input.child_value(self.index);
        self.index += 1;
        let value = entry.child_value(1);
        seed.deserialize(value.as_serializable())
    }

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
        V: de::DeserializeSeed<'de>,
    {
        if self.index >= self.input.n_children() {
            return Ok(None);
        }
        let entry = self.input.child_value(self.index);
        self.index += 1;
        let key = entry.child_value(0);
        let value = entry.child_value(1);
        let key = kseed.deserialize(key.as_serializable())?;
        let value = vseed.deserialize(value.as_serializable())?;
        Ok(Some((key, value)))
    }

    fn size_hint(&self) -> Option<usize> {
        <Self as de::SeqAccess>::size_hint(self)
    }
}
