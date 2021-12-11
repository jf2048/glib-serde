use glib::variant::DictEntry;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

/// A tree node that stores [`glib::VariantTy`]s for enum variants.
#[derive(Clone, Debug)]
pub struct VariantTypeNode<'t> {
    ty: Cow<'t, glib::VariantTy>,
    child_tys: Vec<Cow<'t, VariantTypeNode<'t>>>,
}

impl<'t> VariantTypeNode<'t> {
    pub(crate) const fn new_static(ty: &'t glib::VariantTy) -> Self {
        Self {
            ty: Cow::Borrowed(ty),
            child_tys: Vec::new(),
        }
    }
    pub fn new(
        ty: Cow<'t, glib::VariantTy>,
        child_tys: impl IntoIterator<Item = Cow<'t, VariantTypeNode<'t>>>,
    ) -> Self {
        Self {
            ty,
            child_tys: Vec::from_iter(child_tys),
        }
    }
    pub fn type_(&self) -> &Cow<'t, glib::VariantTy> {
        &self.ty
    }
    pub fn child_types(&self) -> &[Cow<'t, VariantTypeNode<'t>>] {
        &self.child_tys
    }
}

/// An extension of [`StaticVariantType`](glib::StaticVariantType) that can retreive types for enum
/// variants.
pub trait VariantType: glib::StaticVariantType {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        Cow::Owned(VariantTypeNode::new(Self::static_variant_type(), []))
    }
}

impl VariantType for glib::Variant {}
impl VariantType for glib::VariantDict {}
impl VariantType for () {}
impl VariantType for u8 {}
impl VariantType for i16 {}
impl VariantType for u16 {}
impl VariantType for i32 {}
impl VariantType for u32 {}
impl VariantType for i64 {}
impl VariantType for u64 {}
impl VariantType for f64 {}
impl VariantType for bool {}
impl VariantType for String {}
impl VariantType for str {}

impl<'a, T: ?Sized + VariantType> VariantType for &'a T {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        T::variant_type()
    }
}

impl<T: VariantType> VariantType for Option<T> {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        let child_node = T::variant_type();
        let mut builder = glib::GStringBuilder::new("m");
        builder.append(child_node.type_().as_str());
        let ty = glib::VariantType::from_string(builder.into_string()).unwrap();
        Cow::Owned(VariantTypeNode::new(
            Cow::Owned(ty),
            [child_node.to_owned()],
        ))
    }
}

impl<T: VariantType> VariantType for [T] {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        let child_node = T::variant_type();
        let mut builder = glib::GStringBuilder::new("a");
        builder.append(child_node.type_().as_str());
        let ty = glib::VariantType::from_string(builder.into_string()).unwrap();
        Cow::Owned(VariantTypeNode::new(
            Cow::Owned(ty),
            [child_node.to_owned()],
        ))
    }
}

impl<T: VariantType> VariantType for Vec<T> {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        <[T]>::variant_type()
    }
}

impl<A: AsRef<[T]>, T: glib::FixedSizeVariantType + VariantType> VariantType
    for glib::FixedSizeVariantArray<A, T>
{
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        <[T]>::variant_type()
    }
}

impl<K: VariantType, V: VariantType> VariantType for DictEntry<K, V> {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        let key_node = K::variant_type();
        let value_node = V::variant_type();
        let mut builder = glib::GStringBuilder::new("{");
        builder.append(key_node.type_().as_str());
        builder.append(value_node.type_().as_str());
        builder.append_c('}');
        let ty = glib::VariantType::from_string(builder.into_string()).unwrap();
        Cow::Owned(VariantTypeNode::new(
            Cow::Owned(ty),
            [key_node.to_owned(), value_node.to_owned()],
        ))
    }
}

impl<K: VariantType, V: VariantType> VariantType for HashMap<K, V> {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        let child_node = <DictEntry<K, V>>::variant_type();
        let mut builder = glib::GStringBuilder::new("a");
        builder.append(child_node.type_().as_str());
        let ty = glib::VariantType::from_string(builder.into_string()).unwrap();
        Cow::Owned(VariantTypeNode::new(
            Cow::Owned(ty),
            child_node.child_types().to_owned(),
        ))
    }
}

impl<K: VariantType, V: VariantType> VariantType for BTreeMap<K, V> {
    fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
        <HashMap<K, V>>::variant_type()
    }
}

macro_rules! tuple_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+) => {
        $(
            impl<$($name),+> VariantType for ($($name,)+)
            where
                $($name: VariantType,)+
            {
                fn variant_type() -> Cow<'static, VariantTypeNode<'static>> {
                    Cow::Owned(VariantTypeNode::new(
                        <Self as glib::StaticVariantType>::static_variant_type(),
                        [$($name::variant_type()),+],
                    ))
                }
            }
        )+
    }
}

tuple_impls! {
    1 => (0 T0)
    2 => (0 T0 1 T1)
    3 => (0 T0 1 T1 2 T2)
    4 => (0 T0 1 T1 2 T2 3 T3)
    5 => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}
