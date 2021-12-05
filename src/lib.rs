pub use glib;
pub use glib_serde_derive::*;

mod error;
pub use error::*;
mod object_path;
pub use object_path::*;
mod variant;
pub use variant::{Variant, from_variant, to_variant};
mod variant_dict;
pub use variant_dict::*;
mod variant_type;
pub use variant_type::*;

pub mod prelude {
    pub use super::variant::VariantSerializeExt;
}
