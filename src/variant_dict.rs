use serde::ser::SerializeMap;

use crate::{prelude::*, Variant, VariantTypeNode};
use std::borrow::Cow;

#[repr(transparent)]
pub struct VariantDict(glib::VariantDict);

impl VariantDict {
    pub fn new(from_asv: Option<&glib::Variant>) -> Self {
        Self(glib::VariantDict::new(from_asv))
    }
}

impl std::ops::Deref for VariantDict {
    type Target = glib::VariantDict;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for VariantDict {
    fn default() -> Self {
        Self::new(None)
    }
}

impl glib::StaticVariantType for VariantDict {
    fn static_variant_type() -> Cow<'static, glib::VariantTy> {
        <glib::VariantDict as glib::StaticVariantType>::static_variant_type()
    }
}

impl super::VariantType for VariantDict {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        Cow::Owned(VariantTypeNode::new(
            <Self as glib::StaticVariantType>::static_variant_type(),
            [String::variant_type(), Variant::variant_type()],
        ))
    }
}

impl glib::ToVariant for VariantDict {
    fn to_variant(&self) -> glib::Variant {
        self.0.to_variant()
    }
}

impl glib::FromVariant for VariantDict {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        <glib::VariantDict as glib::FromVariant>::from_variant(variant).map(Self)
    }
}

impl From<glib::Variant> for VariantDict {
    fn from(other: glib::Variant) -> Self {
        Self::new(Some(&other))
    }
}

impl From<VariantDict> for glib::VariantDict {
    fn from(dict: VariantDict) -> Self {
        dict.0
    }
}

impl From<glib::VariantDict> for VariantDict {
    fn from(dict: glib::VariantDict) -> Self {
        Self(dict)
    }
}

impl serde::ser::Serialize for VariantDict {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let v = self.end();
        let count = v.n_children();
        let mut seq = serializer.serialize_map(Some(count))?;
        for i in 0..count {
            let entry = v.child_value(i);
            let key = entry.child_value(0);
            let key = key.str().unwrap();
            let value = entry.child_value(1).as_variant().unwrap();
            seq.serialize_entry(&key, &value.as_serializable())?;
        }
        seq.end()
    }
}

impl<'de> serde::de::Deserialize<'de> for VariantDict {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct MapVisitor;

        impl<'de> serde::de::Visitor<'de> for MapVisitor {
            type Value = glib::VariantDict;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VariantDict map")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let dict = glib::VariantDict::new(None);

                while let Some((key, value)) = map.next_entry::<String, Variant>()? {
                    dict.insert_value(&key, &value);
                }

                Ok(dict)
            }
        }

        deserializer.deserialize_map(MapVisitor).map(Into::into)
    }
}
