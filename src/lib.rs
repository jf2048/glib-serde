pub use glib;
pub use glib_serde_derive::*;

mod error;
pub use error::*;
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
    pub use super::variant::VariantSerializeExt;
}
