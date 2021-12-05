use crate::{Error, VariantType, VariantTypeNode, ObjectPath};
use glib::{translate::*, ToVariant, VariantClass, VariantTy, variant::VariantTypeMismatchError};
use serde::{
    ser::{self, SerializeMap, SerializeSeq, SerializeTuple},
    de::{self, Visitor},
    Deserialize,
    Deserializer,
    Serialize,
};
use std::ops::Deref;

pub fn to_variant<T>(value: &T) -> Result<glib::Variant, Error>
where
    T: Serialize + VariantType,
{
    let node = T::variant_type();
    let mut serializer = Serializer::new(&node);
    value.serialize(&mut serializer)
}

pub fn from_variant<'de, T>(variant: &'de glib::Variant) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    T::deserialize(variant.as_serializable())
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Variant(glib::Variant);

impl glib::StaticVariantType for Variant {
    fn static_variant_type() -> std::borrow::Cow<'static, VariantTy> {
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

impl<'de> Deserialize<'de> for Variant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(VariantVisitor).map(|v| v.into())
    }
}

#[inline]
fn try_serialize<T, S>(v: &glib::Variant, serializer: S) -> Result<S::Ok, S::Error>
where
    T: glib::FromVariant + Serialize,
    S: ser::Serializer,
{
    v.try_get::<T>()
        .map_err(|e| ser::Error::custom(e.to_string()))?
        .serialize(serializer)
}

impl Serialize for Variant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let v = self;
        match v.classify() {
            VariantClass::Boolean => try_serialize::<bool, _>(v, serializer),
            VariantClass::Byte => try_serialize::<u8, _>(v, serializer),
            VariantClass::Int16 => try_serialize::<i16, _>(v, serializer),
            VariantClass::Uint16 => try_serialize::<u16, _>(v, serializer),
            VariantClass::Int32 => try_serialize::<i32, _>(v, serializer),
            VariantClass::Uint32 => try_serialize::<i32, _>(v, serializer),
            VariantClass::Int64 => try_serialize::<i64, _>(v, serializer),
            VariantClass::Uint64 => try_serialize::<i64, _>(v, serializer),
            VariantClass::Handle => Err(ser::Error::custom("Handle values not supported")),
            VariantClass::Double => try_serialize::<f64, _>(v, serializer),
            VariantClass::String | VariantClass::ObjectPath | VariantClass::Signature => {
                v.str()
                    .ok_or_else(|| {
                        ser::Error::custom(
                            crate::Error::StrMismatch(v.type_().to_owned())
                        )
                    })?
                    .serialize(serializer)
            },
            VariantClass::Variant => {
                let variant = v.try_get::<glib::Variant>()
                    .map_err(|e| ser::Error::custom(e.to_string()))?;
                variant.as_serializable().serialize(serializer)
            }
            VariantClass::Maybe => {
                let child: Option<glib::Variant> = unsafe {
                    let child = glib::ffi::g_variant_get_maybe(v.to_glib_none().0);
                    if child.is_null() {
                        None
                    } else {
                        Some(from_glib_full(child))
                    }
                };
                match child {
                    Some(child) => serializer.serialize_some(&child.as_serializable()),
                    None => serializer.serialize_none(),
                }
            }
            VariantClass::Array => {
                let count = v.n_children();
                let child_type = v.type_().element();
                if child_type.is_dict_entry() {
                    let mut seq = serializer.serialize_map(Some(count))?;
                    for i in 0..count {
                        let entry = v.child_value(i);
                        let key = entry.child_value(0);
                        let value = entry.child_value(1);
                        seq.serialize_entry(
                            &key.as_serializable(),
                            &value.as_serializable(),
                        )?;
                    }
                    seq.end()
                } else {
                    let mut seq = serializer.serialize_seq(Some(count))?;
                    for i in 0..count {
                        let child = v.child_value(i);
                        seq.serialize_element(&child.as_serializable())?;
                    }
                    seq.end()
                }
            }
            VariantClass::Tuple => {
                let count = v.n_children();
                let mut seq = serializer.serialize_tuple(count)?;
                for i in 0..count {
                    let child = v.child_value(i);
                    seq.serialize_element(&child.as_serializable())?;
                }
                seq.end()
            }
            VariantClass::DictEntry => Err(ser::Error::custom("Dict entry must be inside an array")),
            _ => panic!("Unknown variant type"),
        }
    }
}

pub trait VariantSerializeExt {
    fn as_serializable(&self) -> &Variant;
}

impl VariantSerializeExt for glib::Variant {
    fn as_serializable(&self) -> &Variant {
        unsafe {
            &*(self as *const glib::Variant as *const Variant)
        }
    }
}

#[repr(transparent)]
struct Serializer<'t> {
    node: &'t VariantTypeNode,
}

impl<'t> Serializer<'t> {
    fn new(node: &'t VariantTypeNode) -> Self {
        Self { node }
    }
    fn variant_tag(
        &self,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(VariantTag, Option<&'t VariantTypeNode>), Error> {
        let ty = self.node.type_();
        let (tag_ty, value_ty) = if ty.is_tuple() {
            let tag_ty = ty.first().unwrap();
            let value_ty = &self.node.child_types()[variant_index as usize];
            (tag_ty, Some(&**value_ty))
        } else {
            (std::ops::Deref::deref(ty), None)
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
            _ => return Err(Error::InvalidTag(tag_ty.to_owned()))
        };
        Ok((tag, value_ty))
    }
}

impl<'t> ser::Serializer for &mut Serializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;
    type SerializeSeq = SeqSerializer<'t>;
    type SerializeTuple = TupleSerializer<'t>;
    type SerializeTupleStruct = TupleSerializer<'t>;
    type SerializeTupleVariant = TupleVariantSerializer<'t>;
    type SerializeMap = MapSerializer<'t>;
    type SerializeStruct = TupleSerializer<'t>;
    type SerializeStructVariant = TupleVariantSerializer<'t>;

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
        match self.node.type_().as_str() {
            "o" => ObjectPath::new(v).map(|o| o.to_variant()).map_err(Error::Bool),
            "g" => VariantTy::new(v).map(|t| t.as_str().to_variant()).map_err(Error::Bool),
            "s" => Ok(v.to_variant()),
            _ => Err(Error::StrMismatch(self.node.type_().deref().to_owned()))
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(glib::Variant::array_from_fixed_array(v))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        let ty = &self.node.child_types()[0];
        let variant = unsafe {
            from_glib_none(glib::ffi::g_variant_new_maybe(
                ty.type_().as_ptr() as *const _,
                std::ptr::null_mut(),
            ))
        };
        Ok(variant)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let ty = &self.node.child_types()[0];
        let value = value.serialize(&mut Serializer::new(ty))?;
        let variant = unsafe {
            from_glib_none(glib::ffi::g_variant_new_maybe(
                ty.type_().as_ptr() as *const _,
                value.to_glib_none().0 as *mut glib::ffi::GVariant,
            ))
        };
        Ok(variant)
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
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
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
        Ok(SeqSerializer::new(self.node, len.unwrap_or_default()))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(TupleSerializer::new(self.node, len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(TupleSerializer::new(self.node, len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let (tag, value_ty) = self.variant_tag(variant_index, variant)?;
        Ok(TupleVariantSerializer::new(tag, value_ty.unwrap(), len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer::new(self.node, len.unwrap_or_default()))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(TupleSerializer::new(self.node, len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let (tag, value_ty) = self.variant_tag(variant_index, variant)?;
        Ok(TupleVariantSerializer::new(tag, value_ty.unwrap(), len))
    }
}

fn array_from_iter(ty: &VariantTy, children: impl IntoIterator<Item = glib::Variant>) -> glib::Variant {
    let mut builder = std::mem::MaybeUninit::uninit();
    unsafe {
        glib::ffi::g_variant_builder_init(builder.as_mut_ptr(), ty.to_glib_none().0);
        let mut builder = builder.assume_init();
        for value in children {
            glib::ffi::g_variant_builder_add_value(&mut builder, value.to_glib_none().0);
        }
        from_glib_none(glib::ffi::g_variant_builder_end(&mut builder))
    }
}

struct SeqSerializer<'t> {
    node: &'t VariantTypeNode,
    variants: Vec<glib::Variant>,
}

impl<'t> SeqSerializer<'t> {
    fn new(node: &'t VariantTypeNode, size: usize) -> Self {
        Self {
            node,
            variants: Vec::with_capacity(size),
        }
    }
}

impl<'t> ser::SerializeSeq for SeqSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let node = &self.node.child_types()[0];
        self.variants.push(value.serialize(&mut Serializer::new(node))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(array_from_iter(&self.node.type_(), self.variants))
    }
}

struct TupleSerializer<'t> {
    node: &'t VariantTypeNode,
    index: usize,
    variants: Vec<glib::Variant>,
}

impl<'t> TupleSerializer<'t> {
    fn new(node: &'t VariantTypeNode, size: usize) -> Self {
        Self {
            node,
            index: 0,
            variants: Vec::with_capacity(size),
        }
    }
}

impl<'t> ser::SerializeTuple for TupleSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let ty = &self.node.child_types()[self.index];
        self.variants
            .push(value.serialize(&mut Serializer::new(ty))?);
        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(glib::Variant::tuple_from_iter(self.variants.into_iter()))
    }
}

impl<'t> ser::SerializeTupleStruct for TupleSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeTuple>::end(self)
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

struct TupleVariantSerializer<'t> {
    tag: VariantTag,
    inner: TupleSerializer<'t>,
}

impl<'t> TupleVariantSerializer<'t> {
    fn new(tag: VariantTag, node: &'t VariantTypeNode, size: usize) -> Self {
        Self {
            tag,
            inner: TupleSerializer::new(node, size),
        }
    }
}

impl<'t> ser::SerializeTupleVariant for TupleVariantSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.inner.serialize_element(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok((self.tag, self.inner.end()?).to_variant())
    }
}

struct MapSerializer<'t> {
    node: &'t VariantTypeNode,
    key: Option<glib::Variant>,
    variants: Vec<glib::Variant>,
}

impl<'t> MapSerializer<'t> {
    fn new(node: &'t VariantTypeNode, size: usize) -> Self {
        Self {
            node,
            key: None,
            variants: Vec::with_capacity(size),
        }
    }
}

impl<'t> ser::SerializeMap for MapSerializer<'t> {
    type Ok = glib::Variant;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        assert!(self.key.is_none());
        let key_node = &self.node.child_types()[0];
        self.key.replace(key.serialize(&mut Serializer::new(key_node))?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let value_node = &self.node.child_types()[1];
        let value = value.serialize(&mut Serializer::new(value_node))?;
        let variant = unsafe {
            from_glib_none(glib::ffi::g_variant_new_dict_entry(
                self.key.take().unwrap().to_glib_none().0,
                value.to_glib_none().0,
            ))
        };
        self.variants.push(variant);
        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        assert!(self.key.is_none());
        let key_node = &self.node.child_types()[0];
        let value_node = &self.node.child_types()[1];
        let key = key.serialize(&mut Serializer::new(key_node))?;
        let value = value.serialize(&mut Serializer::new(value_node))?;
        let variant = unsafe {
            from_glib_none(glib::ffi::g_variant_new_dict_entry(
                key.to_glib_none().0,
                value.to_glib_none().0,
            ))
        };
        self.variants.push(variant);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(array_from_iter(&self.node.type_(), self.variants))
    }
}

impl<'t> ser::SerializeStruct for TupleSerializer<'t> {
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

impl<'t> ser::SerializeStructVariant for TupleVariantSerializer<'t> {
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
        Ok((self.tag, self.inner.end()?).to_variant())
    }
}

struct VariantVisitor;

impl<'de> Visitor<'de> for VariantVisitor {
    type Value = glib::Variant;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any valid GVariant value")
    }

    #[inline]
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(value.to_variant())
    }

    #[inline]
    fn visit_i8<E: de::Error>(self, v: i8) -> Result<Self::Value, E> {
        self.visit_i16(v as i16)
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

    #[inline]
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(v.to_variant())
    }

    #[inline]
    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(glib::Variant::array_from_fixed_array(v))
    }

    #[inline]
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(<Option<()>>::None.to_variant())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Variant = Deserialize::deserialize(deserializer)?;
        Ok(unsafe {
            from_glib_none(glib::ffi::g_variant_new_maybe(
                std::ptr::null(),
                value.to_glib_none().0 as *mut glib::ffi::GVariant,
            ))
        })
    }

    #[inline]
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(().to_variant())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        todo!()
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        todo!()
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        todo!()
    }
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
            },
            VariantClass::Variant => {
                let variant = self.try_get::<glib::Variant>()?;
                variant.as_serializable().deserialize_any(visitor)
            },
            VariantClass::Maybe => self.deserialize_option(visitor),
            VariantClass::Array => {
                if self.type_().element().is_dict_entry() {
                    self.deserialize_map(visitor)
                } else {
                    self.deserialize_seq(visitor)
                }
            },
            VariantClass::Tuple => {
                self.deserialize_tuple(self.n_children(), visitor)
            },
            _ => {
                Err(Error::UnsupportedType(self.type_().to_owned()))
            },
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

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f32(self.try_get::<f64>()? as f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_f64(self.try_get()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self.str().ok_or_else(|| Error::StrMismatch(self.type_().to_owned()))?;
        let c = s.chars().next().ok_or_else(|| Error::ExpectedChar(s.to_owned()))?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self.str().ok_or_else(|| Error::StrMismatch(self.type_().to_owned()))?;
        visitor.visit_str(s)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s = self.str().ok_or_else(|| Error::StrMismatch(self.type_().to_owned()))?;
        visitor.visit_string(s.to_owned())
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bytes(self.fixed_array()?)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_byte_buf(self.fixed_array()?.to_owned())
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let child: Option<glib::Variant> = unsafe {
            let child = glib::ffi::g_variant_get_maybe(self.to_glib_none().0);
            if child.is_null() {
                None
            } else {
                Some(from_glib_full(child))
            }
        };
        match child {
            Some(child) => {
                visitor.visit_some(child.as_serializable())
            },
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
        check_type(&self, VariantTy::ARRAY)?;
        visitor.visit_seq(ContainerDeserializer::new(self))
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error> {
        check_type(&self, VariantTy::TUPLE)?;
        if self.n_children() != len {
            return Err(Error::LengthMismatch {
                actual: self.n_children(),
                expected: len
            });
        }
        visitor.visit_seq(ContainerDeserializer::new(self))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        check_type(&self, VariantTy::DICTIONARY)?;
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
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        todo!()
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        todo!()
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_any(visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

#[inline]
fn check_type(
    variant: &glib::Variant,
    ty: &VariantTy
) -> Result<(), VariantTypeMismatchError> {
    let is_type: bool = unsafe {
        from_glib(glib::ffi::g_variant_is_of_type(
            variant.to_glib_none().0,
            ty.to_glib_none().0,
        ))
    };
    if is_type {
        Ok(())
    } else {
        Err(
            VariantTypeMismatchError::new(
                variant.type_().to_owned(),
                ty.to_owned(),
            )
        )
    }
}

struct ContainerDeserializer<'v> {
    input: &'v Variant,
    index: usize
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
        T: de::DeserializeSeed<'de>
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
        vseed: V
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

#[cfg(test)]
mod tests {
    use glib::VariantTy;
    use crate::{ObjectPath, VariantDict};
    #[test]
    fn basic_types() {
        let variant = super::to_variant(&true).unwrap();
        assert_eq!(variant.type_(), VariantTy::BOOLEAN);
        assert_eq!(variant.to_string(), "true");

        let variant = super::to_variant(&-3i16).unwrap();
        assert_eq!(variant.type_(), VariantTy::INT16);
        assert_eq!(variant.to_string(), "-3");

        let variant = super::to_variant(&-4i32).unwrap();
        assert_eq!(variant.type_(), VariantTy::INT32);
        assert_eq!(variant.to_string(), "-4");

        let variant = super::to_variant(&-5i64).unwrap();
        assert_eq!(variant.type_(), VariantTy::INT64);
        assert_eq!(variant.to_string(), "-5");

        let variant = super::to_variant(&6u8).unwrap();
        assert_eq!(variant.type_(), VariantTy::BYTE);
        assert_eq!(variant.to_string(), "0x06");

        let variant = super::to_variant(&7u16).unwrap();
        assert_eq!(variant.type_(), VariantTy::UINT16);
        assert_eq!(variant.to_string(), "7");

        let variant = super::to_variant(&8u32).unwrap();
        assert_eq!(variant.type_(), VariantTy::UINT32);
        assert_eq!(variant.to_string(), "8");

        let variant = super::to_variant(&9u64).unwrap();
        assert_eq!(variant.type_(), VariantTy::UINT64);
        assert_eq!(variant.to_string(), "9");

        let variant = super::to_variant(&10.1f64).unwrap();
        assert_eq!(variant.type_(), VariantTy::DOUBLE);
        assert_eq!(variant.to_string(), "10.1");

        let variant = super::to_variant(&"123").unwrap();
        assert_eq!(variant.type_(), VariantTy::STRING);
        assert_eq!(variant.to_string(), "'123'");

        let variant = super::to_variant(&String::from("124")).unwrap();
        assert_eq!(variant.type_(), VariantTy::STRING);
        assert_eq!(variant.to_string(), "'124'");

        let variant = super::to_variant(&ObjectPath::new("/com/org/Test").unwrap()).unwrap();
        assert_eq!(variant.type_(), VariantTy::OBJECT_PATH);
        assert_eq!(variant.to_string(), "'/com/org/Test'");

        let dict = VariantDict::new(None);
        dict.insert("a", &200i32);
        dict.insert("b", &(300i64, 400.4f64));
        let variant = super::to_variant(&dict).unwrap();
        assert_eq!(variant.type_(), VariantTy::VARDICT);
        assert_eq!(variant.to_string(), "[]");
    }
}
