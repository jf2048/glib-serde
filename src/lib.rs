pub use glib;
pub use glib_serde_derive::*;
pub use serde;

mod enums;
pub use enums::*;
mod error;
pub use error::*;
mod flags;
pub use flags::*;
mod object_path;
pub use object_path::*;
mod signature;
pub use signature::*;
mod variant;
pub use variant::{from_variant, to_variant, Variant};
mod variant_builder;
use variant_builder::*;
mod variant_dict;
pub use variant_dict::*;
mod variant_type;
pub use variant_type::*;

pub mod prelude {
    pub use super::variant::{GlibVariantExt, VariantSerializeExt};

    pub trait ToVariantExt {
        fn serialize_to_variant(&self) -> glib::Variant;
    }

    impl<T: serde::Serialize + super::VariantType> ToVariantExt for T {
        fn serialize_to_variant(&self) -> glib::Variant {
            super::to_variant(self).unwrap()
        }
    }

    pub trait FromVariantExt<'t, T> {
        fn deserialize_from_variant(variant: &'t glib::Variant) -> Option<T>;
    }

    impl<'de, T: serde::Deserialize<'de>> FromVariantExt<'de, T> for T {
        fn deserialize_from_variant(variant: &'de glib::Variant) -> Option<T> {
            super::from_variant(variant).ok()
        }
    }
}
