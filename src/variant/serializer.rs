// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

use super::GlibVariantExt;
use crate::{object_path, signature, Error, ObjectPath, Signature, VariantType, VariantTypeNode};
use glib::{ToVariant, VariantTy};
use serde::{
    ser::{self, SerializeTuple},
    Serialize,
};
use std::{borrow::Cow, ops::Deref};

/// Serializes `T` into a [`glib::Variant`](struct@glib::Variant).
pub fn to_variant<T>(value: &T) -> Result<glib::Variant, Error>
where
    T: Serialize + VariantType,
{
    let ty = T::variant_type();
    let serializer = Serializer::new(&ty);
    value.serialize(serializer)
}

#[repr(transparent)]
struct Serializer<'t, 'n> {
    node: &'t VariantTypeNode<'n>,
}

fn child_type_or_default<'t, 'n>(
    node: &'t VariantTypeNode<'n>,
    index: usize,
) -> Cow<'t, VariantTypeNode<'n>> {
    node.child_types()
        .get(index as usize)
        .map(|t| Cow::Borrowed(t.deref()))
        .or_else(|| {
            let ty = node.type_().deref();
            if ty.is_array() || ty.is_maybe() {
                let elem = ty.element();
                if elem.is_dict_entry() {
                    if index == 0 {
                        Some(elem.key())
                    } else if index == 1 {
                        Some(elem.value())
                    } else {
                        None
                    }
                } else if index == 0 {
                    Some(elem)
                } else {
                    None
                }
            } else if ty.is_tuple() {
                let mut i = 0;
                let mut iter = ty.first();
                while let Some(child) = iter {
                    if index == i {
                        break;
                    }
                    iter = child.next();
                    i += 1;
                }
                iter
            } else {
                None
            }
            .map(|t| Cow::Owned(VariantTypeNode::new(t.to_owned().into(), [])))
        })
        .unwrap_or_else(|| Cow::Owned(VariantTypeNode::new(Cow::Borrowed(VariantTy::ANY), [])))
}

impl<'t, 'n> Serializer<'t, 'n> {
    fn new(node: &'t VariantTypeNode<'n>) -> Self {
        Self { node }
    }
    fn variant_tag(
        &self,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(VariantTag, Option<Cow<'t, VariantTypeNode<'n>>>), Error> {
        let ty = self.node.type_();
        let (tag_ty, value_ty) = if ty.is_tuple() {
            let tag_ty = ty
                .first()
                .ok_or_else(|| Error::UnsupportedType(ty.deref().to_owned()))?;
            let value_node = child_type_or_default(self.node, variant_index as usize);
            (tag_ty, Some(value_node))
        } else {
            (ty.deref(), None)
        };
        let tag = match tag_ty.as_str() {
            "s" => VariantTag::Str(variant.to_owned()),
            "n" => VariantTag::I16(variant_index.try_into()?),
            "i" => VariantTag::I32(variant_index.try_into()?),
            "x" => VariantTag::I64(variant_index as i64),
            "y" => VariantTag::U8(variant_index.try_into()?),
            "q" => VariantTag::U16(variant_index.try_into()?),
            "u" => VariantTag::U32(variant_index),
            "t" => VariantTag::U64(variant_index as u64),
            _ => return Err(Error::InvalidTag(tag_ty.to_owned())),
        };
        Ok((tag, value_ty))
    }
}

impl<'t, 'n> ser::Serializer for Serializer<'t, 'n> {
    type Ok = glib::Variant;
    type Error = Error;
    type SerializeSeq = SeqSerializer<'t, 'n>;
    type SerializeTuple = TupleSerializer<'t, 'n>;
    type SerializeTupleStruct = TupleSerializer<'t, 'n>;
    type SerializeTupleVariant = TupleVariantSerializer<'t, 'n>;
    type SerializeMap = MapSerializer<'t, 'n>;
    type SerializeStruct = TupleSerializer<'t, 'n>;
    type SerializeStructVariant = TupleVariantSerializer<'t, 'n>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i16(v as i16)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    serde::serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            let v = v as u128;
            let buf = [(v >> 64) as i64, v as i64];
            Ok(glib::Variant::array_from_fixed_array(&buf))
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    serde::serde_if_integer128! {
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            let buf = [(v >> 64) as u64, v as u64];
            Ok(glib::Variant::array_from_fixed_array(&buf))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_variant())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let ty = self.node.type_();
        match ty.as_str() {
            "o" => ObjectPath::new(v)
                .map(|o| o.to_variant())
                .map_err(Error::Bool),
            "g" => Signature::new(v)
                .map(|g| g.to_variant())
                .map_err(Error::Bool),
            "s" | "*" => Ok(v.to_variant()),
            _ => Err(Error::StrMismatch(ty.deref().to_owned())),
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(glib::Variant::array_from_fixed_array(v))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        let ty = child_type_or_default(self.node, 0);
        Ok(glib::Variant::from_none(ty.type_()))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let ty = child_type_or_default(self.node, 0);
        let value = value.serialize(Serializer::new(&ty))?;
        value.is_of_type(ty.type_())?;
        Ok(glib::Variant::from_some(&value))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(().to_variant())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(().to_variant())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let (tag, value_ty) = self.variant_tag(variant_index, variant)?;
        if value_ty.is_some() {
            Ok((tag, ().to_variant()).to_variant())
        } else {
            Ok(tag.to_variant())
        }
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        static OBJECT_PATH_NODE: VariantTypeNode<'static> =
            VariantTypeNode::new_static(VariantTy::OBJECT_PATH);
        static SIGNATURE_NODE: VariantTypeNode<'static> =
            VariantTypeNode::new_static(VariantTy::SIGNATURE);
        match name {
            object_path::STRUCT_NAME => value.serialize(Serializer::new(&OBJECT_PATH_NODE)),
            signature::STRUCT_NAME => value.serialize(Serializer::new(&SIGNATURE_NODE)),
            _ => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let (tag, _) = self.variant_tag(variant_index, variant)?;
        Ok((tag, value.serialize(self)?).to_variant())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.unwrap_or_default();
        let ty = self.node.type_();
        if ty.is_array() {
            match ty.element().as_str() {
                "y" => Ok(SeqSerializer::new_u8(len)),
                "q" => Ok(SeqSerializer::new_u16(len)),
                "u" => Ok(SeqSerializer::new_u32(len)),
                "t" => Ok(SeqSerializer::new_u64(len)),
                "n" => Ok(SeqSerializer::new_i16(len)),
                "i" => Ok(SeqSerializer::new_i32(len)),
                "x" => Ok(SeqSerializer::new_i64(len)),
                "d" => Ok(SeqSerializer::new_f64(len)),
                "b" => Ok(SeqSerializer::new_bool(len)),
                _ => Ok(SeqSerializer::new(self.node, len)),
            }
        } else {
            Ok(SeqSerializer::new(self.node, len))
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(TupleSerializer::new(Cow::Borrowed(self.node), "", len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(TupleSerializer::new(Cow::Borrowed(self.node), name, len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let (tag, value_ty) = self.variant_tag(variant_index, variant)?;
        let value_ty =
            value_ty.ok_or_else(|| Error::UnsupportedType(self.node.type_().deref().to_owned()))?;
        Ok(TupleVariantSerializer::new(tag, value_ty, name, len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer::new(self.node, len.unwrap_or_default()))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(TupleSerializer::new(Cow::Borrowed(self.node), name, len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let (tag, value_ty) = self.variant_tag(variant_index, variant)?;
        let value_ty =
            value_ty.ok_or_else(|| Error::UnsupportedType(self.node.type_().deref().to_owned()))?;
        Ok(TupleVariantSerializer::new(tag, value_ty, name, len))
    }
}

enum SeqSerializer<'t, 'n> {
    Variant {
        node: &'t VariantTypeNode<'n>,
        child_node: Cow<'t, VariantTypeNode<'n>>,
        variants: Vec<glib::Variant>,
    },
    U8 {
        values: Vec<u8>,
    },
    U16 {
        values: Vec<u16>,
    },
    U32 {
        values: Vec<u32>,
    },
    U64 {
        values: Vec<u64>,
    },
    I16 {
        values: Vec<i16>,
    },
    I32 {
        values: Vec<i32>,
    },
    I64 {
        values: Vec<i64>,
    },
    F64 {
        values: Vec<f64>,
    },
    Bool {
        values: Vec<bool>,
    },
}

impl<'t, 'n> SeqSerializer<'t, 'n> {
    fn new(node: &'t VariantTypeNode<'n>, size: usize) -> Self {
        let child_node = child_type_or_default(node, 0);
        Self::Variant {
            node,
            child_node,
            variants: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_u8(size: usize) -> Self {
        Self::U8 {
            values: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_u16(size: usize) -> Self {
        Self::U16 {
            values: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_u32(size: usize) -> Self {
        Self::U32 {
            values: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_u64(size: usize) -> Self {
        Self::U64 {
            values: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_i16(size: usize) -> Self {
        Self::I16 {
            values: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_i32(size: usize) -> Self {
        Self::I32 {
            values: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_i64(size: usize) -> Self {
        Self::I64 {
            values: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_f64(size: usize) -> Self {
        Self::F64 {
            values: Vec::with_capacity(size),
        }
    }
    #[inline]
    fn new_bool(size: usize) -> Self {
        Self::Bool {
            values: Vec::with_capacity(size),
        }
    }
}

impl<'t, 'n> ser::SerializeSeq for SeqSerializer<'t, 'n> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_element<S: ?Sized>(&mut self, value: &S) -> Result<(), Self::Error>
    where
        S: Serialize,
    {
        match self {
            Self::Variant {
                node: _,
                child_node,
                variants,
            } => {
                let child_node = child_node.clone();
                variants.push(value.serialize(Serializer::new(&child_node))?);
            }
            Self::U8 { values } => {
                values.push(value.serialize(U64Serializer)? as u8);
            }
            Self::U16 { values } => {
                values.push(value.serialize(U64Serializer)? as u16);
            }
            Self::U32 { values } => {
                values.push(value.serialize(U64Serializer)? as u32);
            }
            Self::U64 { values } => {
                values.push(value.serialize(U64Serializer)?);
            }
            Self::I16 { values } => {
                values.push(value.serialize(U64Serializer)? as i16);
            }
            Self::I32 { values } => {
                values.push(value.serialize(U64Serializer)? as i32);
            }
            Self::I64 { values } => {
                values.push(value.serialize(U64Serializer)? as i64);
            }
            Self::F64 { values } => {
                values.push(f64::from_bits(value.serialize(U64Serializer)?));
            }
            Self::Bool { values } => {
                values.push(value.serialize(U64Serializer)? != 0);
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Variant {
                node,
                child_node: _,
                variants,
            } => Ok(glib::Variant::array_from_variant_iter(
                node.type_(),
                variants,
            )),
            Self::U8 { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
            Self::U16 { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
            Self::U32 { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
            Self::U64 { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
            Self::I16 { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
            Self::I32 { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
            Self::I64 { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
            Self::F64 { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
            Self::Bool { values } => Ok(glib::Variant::array_from_fixed_array(&values)),
        }
    }
}

struct TupleSerializer<'t, 'n> {
    node: Cow<'t, VariantTypeNode<'n>>,
    name: &'static str,
    index: usize,
    variants: Vec<glib::Variant>,
}

impl<'t, 'n> TupleSerializer<'t, 'n> {
    fn new(node: Cow<'t, VariantTypeNode<'n>>, name: &'static str, size: usize) -> Self {
        Self {
            node,
            name,
            index: 0,
            variants: Vec::with_capacity(size),
        }
    }
}

impl<'t, 'n> ser::SerializeTuple for TupleSerializer<'t, 'n> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let variant = if self.name == super::STRUCT_NAME && self.index == 1 {
            let ty = glib::VariantType::new(
                self.variants[0]
                    .str()
                    .ok_or_else(|| Error::StrMismatch(self.variants[0].type_().to_owned()))?,
            )?;
            let node = VariantTypeNode::new(ty.into(), []);
            value.serialize(Serializer::new(&node))?
        } else {
            let node = child_type_or_default(&self.node, self.index);
            value.serialize(Serializer::new(&node))?
        };
        self.variants.push(variant);
        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.name == super::STRUCT_NAME {
            assert_eq!(self.variants.len(), 2);
            assert_eq!(
                self.variants[0]
                    .str()
                    .ok_or_else(|| Error::StrMismatch(self.variants[0].type_().to_owned()))?,
                self.variants[1].type_().as_str()
            );
            let mut variants = self.variants;
            Ok(variants.remove(1).to_variant())
        } else {
            Ok(glib::Variant::tuple_from_iter(self.variants.into_iter()))
        }
    }
}

impl<'t, 'n> ser::SerializeTupleStruct for TupleSerializer<'t, 'n> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeTuple::end(self)
    }
}

enum VariantTag {
    Str(String),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}

impl ToVariant for VariantTag {
    fn to_variant(&self) -> glib::Variant {
        match self {
            Self::Str(s) => s.to_variant(),
            Self::I16(i) => i.to_variant(),
            Self::I32(i) => i.to_variant(),
            Self::I64(i) => i.to_variant(),
            Self::U8(u) => u.to_variant(),
            Self::U16(u) => u.to_variant(),
            Self::U32(u) => u.to_variant(),
            Self::U64(u) => u.to_variant(),
        }
    }
}

struct TupleVariantSerializer<'t, 'n> {
    tag: VariantTag,
    inner: TupleSerializer<'t, 'n>,
}

impl<'t, 'n> TupleVariantSerializer<'t, 'n> {
    fn new(
        tag: VariantTag,
        node: Cow<'t, VariantTypeNode<'n>>,
        name: &'static str,
        size: usize,
    ) -> Self {
        Self {
            tag,
            inner: TupleSerializer::new(node, name, size),
        }
    }
}

impl<'t, 'n> ser::SerializeTupleVariant for TupleVariantSerializer<'t, 'n> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.tag, SerializeTuple::end(self.inner)?).to_variant())
    }
}

struct MapSerializer<'t, 'n> {
    node: &'t VariantTypeNode<'n>,
    key: Option<glib::Variant>,
    variants: Vec<glib::Variant>,
}

impl<'t, 'n> MapSerializer<'t, 'n> {
    fn new(node: &'t VariantTypeNode<'n>, size: usize) -> Self {
        Self {
            node,
            key: None,
            variants: Vec::with_capacity(size),
        }
    }
}

impl<'t, 'n> ser::SerializeMap for MapSerializer<'t, 'n> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        assert!(self.key.is_none());
        let key_node = child_type_or_default(self.node, 0);
        self.key.replace(key.serialize(Serializer::new(&key_node))?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value_node = child_type_or_default(self.node, 1);
        let value = value.serialize(Serializer::new(&value_node))?;
        let variant = glib::Variant::from_dict_entry(&self.key.take().unwrap(), &value);
        self.variants.push(variant);
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        assert!(self.key.is_none());
        let key_node = child_type_or_default(self.node, 0);
        let value_node = child_type_or_default(self.node, 1);
        let key = key.serialize(Serializer::new(&key_node))?;
        let value = value.serialize(Serializer::new(&value_node))?;
        let variant = glib::Variant::from_dict_entry(&key, &value);
        self.variants.push(variant);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let ty = self.node.type_();
        Ok(glib::Variant::array_from_variant_iter(ty, self.variants))
    }
}

impl<'t, 'n> ser::SerializeStruct for TupleSerializer<'t, 'n> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeTuple>::end(self)
    }
}

impl<'t, 'n> ser::SerializeStructVariant for TupleVariantSerializer<'t, 'n> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.tag, SerializeTuple::end(self.inner)?).to_variant())
    }
}

struct U64Serializer;

impl ser::Serializer for U64Serializer {
    type Ok = u64;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v as u64)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom("Invalid type: i8".into()))
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v as u64)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v as u64)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v as u64)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v as u64)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v as u64)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v as u64)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v as u64)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom("Invalid type: f32".into()))
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_bits())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom("Invalid type: char".into()))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom("Invalid type: &str".into()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom("Invalid type: &[u8]".into()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom("Invalid type: None".into()))
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::Custom("Invalid type: Some(_)".into()))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom("Invalid type: ()".into()))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom(format!("Invalid type: {}()", name)))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom(format!(
            "Invalid type: {}::{}()",
            name, variant
        )))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::Custom(format!("Invalid type: {}(_)", name)))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::Custom(format!(
            "Invalid type: {}::{}(_)",
            name, variant
        )))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::Custom("Invalid type: sequence".into()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::Custom("Invalid type: tuple".into()))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::Custom(format!("Invalid type: tuple {}", name)))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::Custom(format!(
            "Invalid type: tuple {}::{}",
            name, variant
        )))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::Custom("Invalid type: map".into()))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::Custom(format!("Invalid type: {} {{ }}", name)))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::Custom(format!(
            "Invalid type: {}::{} {{ }}",
            name, variant
        )))
    }
}

impl ser::SerializeSeq for U64Serializer {
    type Ok = u64;
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl ser::SerializeTuple for U64Serializer {
    type Ok = u64;
    type Error = Error;
    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl ser::SerializeTupleStruct for U64Serializer {
    type Ok = u64;
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl ser::SerializeTupleVariant for U64Serializer {
    type Ok = u64;
    type Error = Error;
    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl ser::SerializeMap for U64Serializer {
    type Ok = u64;
    type Error = Error;
    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl ser::SerializeStruct for U64Serializer {
    type Ok = u64;
    type Error = Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
impl ser::SerializeStructVariant for U64Serializer {
    type Ok = u64;
    type Error = Error;
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
