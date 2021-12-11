use super::{GlibVariantExt, Variant};
use crate::{object_path, signature};
use glib::{VariantClass, VariantTy};
use serde::{
    ser::{self, SerializeMap, SerializeSeq, SerializeTuple, SerializeTupleStruct},
    Serialize,
};

struct VariantSerializeInput<'t>(&'t Variant);

impl Serialize for Variant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut tuple = serializer.serialize_tuple_struct(super::STRUCT_NAME, 2)?;
        tuple.serialize_field(self.type_().as_str())?;
        tuple.serialize_field(&VariantSerializeInput(self))?;
        tuple.end()
    }
}

impl<'t> Serialize for VariantSerializeInput<'t> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        #[inline]
        fn try_serialize<T, S>(v: &glib::Variant, serializer: S) -> Result<S::Ok, S::Error>
        where
            T: glib::FromVariant + Serialize,
            S: ser::Serializer,
        {
            v.try_get::<T>()
                .map_err(ser::Error::custom)?
                .serialize(serializer)
        }

        let v = self.0;
        match v.classify() {
            VariantClass::Boolean => try_serialize::<bool, _>(v, serializer),
            VariantClass::Byte => try_serialize::<u8, _>(v, serializer),
            VariantClass::Int16 => try_serialize::<i16, _>(v, serializer),
            VariantClass::Uint16 => try_serialize::<u16, _>(v, serializer),
            VariantClass::Int32 => try_serialize::<i32, _>(v, serializer),
            VariantClass::Uint32 => try_serialize::<u32, _>(v, serializer),
            VariantClass::Int64 => try_serialize::<i64, _>(v, serializer),
            VariantClass::Uint64 => try_serialize::<u64, _>(v, serializer),
            VariantClass::Handle => Err(ser::Error::custom("HANDLE values not supported")),
            VariantClass::Double => try_serialize::<f64, _>(v, serializer),
            VariantClass::String => v.str().unwrap().serialize(serializer),
            VariantClass::ObjectPath => {
                serializer.serialize_newtype_struct(object_path::STRUCT_NAME, v.str().unwrap())
            }
            VariantClass::Signature => {
                serializer.serialize_newtype_struct(signature::STRUCT_NAME, v.str().unwrap())
            }
            VariantClass::Variant => v
                .as_variant()
                .unwrap()
                .as_serializable()
                .serialize(serializer),
            VariantClass::Maybe => match v.maybe().unwrap() {
                Some(inner) => serializer.serialize_some(&VariantSerializeInput(&inner.into())),
                None => serializer.serialize_none(),
            },
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
                            &VariantSerializeInput(&key.into()),
                            &VariantSerializeInput(&value.into()),
                        )?;
                    }
                    seq.end()
                } else if child_type == VariantTy::BYTE {
                    serializer.serialize_bytes(v.fixed_array().map_err(ser::Error::custom)?)
                } else {
                    let mut seq = serializer.serialize_seq(Some(count))?;
                    for i in 0..count {
                        let child = v.child_value(i);
                        seq.serialize_element(&VariantSerializeInput(&child.into()))?;
                    }
                    seq.end()
                }
            }
            VariantClass::Tuple => {
                let count = v.n_children();
                if count > 0 {
                    let mut seq = serializer.serialize_tuple(count)?;
                    for i in 0..count {
                        let child = v.child_value(i);
                        seq.serialize_element(&VariantSerializeInput(&child.into()))?;
                    }
                    seq.end()
                } else {
                    serializer.serialize_unit()
                }
            }
            VariantClass::DictEntry => Err(ser::Error::custom("DICT_ENTRY values not supported")),
            _ => panic!("Unknown variant type"),
        }
    }
}
