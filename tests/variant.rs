use std::collections::HashMap;

use glib::{ToVariant, VariantTy};
use glib_serde::{prelude::*, to_variant, ObjectPath, Signature, VariantDict};

#[test]
fn serialize_basic_types() {
    let variant = to_variant(&true).unwrap();
    assert_eq!(variant.type_(), VariantTy::BOOLEAN);
    assert_eq!(variant.to_string(), "true");

    let variant = to_variant(&-3i16).unwrap();
    assert_eq!(variant.type_(), VariantTy::INT16);
    assert_eq!(variant.to_string(), "-3");

    let variant = to_variant(&-4i32).unwrap();
    assert_eq!(variant.type_(), VariantTy::INT32);
    assert_eq!(variant.to_string(), "-4");

    let variant = to_variant(&-5i64).unwrap();
    assert_eq!(variant.type_(), VariantTy::INT64);
    assert_eq!(variant.to_string(), "-5");

    let variant = to_variant(&6u8).unwrap();
    assert_eq!(variant.type_(), VariantTy::BYTE);
    assert_eq!(variant.to_string(), "0x06");

    let variant = to_variant(&7u16).unwrap();
    assert_eq!(variant.type_(), VariantTy::UINT16);
    assert_eq!(variant.to_string(), "7");

    let variant = to_variant(&8u32).unwrap();
    assert_eq!(variant.type_(), VariantTy::UINT32);
    assert_eq!(variant.to_string(), "8");

    let variant = to_variant(&9u64).unwrap();
    assert_eq!(variant.type_(), VariantTy::UINT64);
    assert_eq!(variant.to_string(), "9");

    let variant = to_variant(&10.1f64).unwrap();
    assert_eq!(variant.type_(), VariantTy::DOUBLE);
    assert_eq!(variant.to_string(), "10.1");

    let variant = to_variant(&"123").unwrap();
    assert_eq!(variant.type_(), VariantTy::STRING);
    assert_eq!(variant.to_string(), "'123'");

    let variant = to_variant(&String::from("124")).unwrap();
    assert_eq!(variant.type_(), VariantTy::STRING);
    assert_eq!(variant.to_string(), "'124'");

    let variant = to_variant(&ObjectPath::new("/com/org/Test").unwrap()).unwrap();
    assert_eq!(variant.type_(), VariantTy::OBJECT_PATH);
    assert_eq!(variant.to_string(), "'/com/org/Test'");

    let variant = to_variant(&Signature::new("(asgva(in)a{sb})").unwrap()).unwrap();
    assert_eq!(variant.type_(), VariantTy::SIGNATURE);
    assert_eq!(variant.to_string(), "'(asgva(in)a{sb})'");
}

#[test]
fn serialize_container_types() {
    let variant = to_variant(&Some("Hello")).unwrap();
    assert_eq!(variant.type_().as_str(), "ms");
    assert_eq!(variant.to_string(), "'Hello'");

    let variant = to_variant(&Option::<String>::None).unwrap();
    assert_eq!(variant.type_().as_str(), "ms");
    assert_eq!(variant.to_string(), "nothing");

    let variant = to_variant(&vec![1i32, 2, 3]).unwrap();
    assert_eq!(variant.type_().as_str(), "ai");
    assert_eq!(variant.to_string(), "[1, 2, 3]");

    let variant = to_variant(&HashMap::from([(2u64, "World")])).unwrap();
    assert_eq!(variant.type_().as_str(), "a{ts}");
    assert_eq!(variant.to_string(), "{2: 'World'}");

    let dict = VariantDict::new(None);
    dict.insert("a", &200i32);
    dict.insert("b", &(300i64, 400.5f64));
    dict.insert("c", &(300i64, 500.5f64));
    let variant = to_variant(&dict).unwrap();
    assert_eq!(variant.type_(), VariantTy::VARDICT);
    assert_eq!(
        variant.to_string(),
        "{'b': <(int64 300, 400.5)>, 'a': <200>, 'c': <(int64 300, 500.5)>}"
    );

    let variant = to_variant("Hello".to_variant().as_serializable()).unwrap();
    assert_eq!(variant.type_(), VariantTy::VARIANT);
    assert_eq!(variant.to_string(), "<'Hello'>");

    let variant = to_variant(
        ObjectPath::new("/com/org/Test")
            .unwrap()
            .to_variant()
            .as_serializable(),
    )
    .unwrap();
    assert_eq!(variant.type_(), VariantTy::VARIANT);
    assert_eq!(variant.to_string(), "<objectpath '/com/org/Test'>");

    let variant = to_variant(
        Signature::new("a(is)")
            .unwrap()
            .to_variant()
            .as_serializable(),
    )
    .unwrap();
    assert_eq!(variant.type_(), VariantTy::VARIANT);
    assert_eq!(variant.to_string(), "<signature 'a(is)'>");

    dict.insert("a", &"abc");
    dict.insert("b", &("hello", "world"));
    let variant = to_variant(&dict.to_variant().as_serializable()).unwrap();
    assert_eq!(variant.type_(), VariantTy::VARIANT);
    assert_eq!(
        variant.to_string(),
        "<{'b': <('hello', 'world')>, 'a': <'abc'>}>"
    );

    let variant = to_variant(Some("Hello").to_variant().as_serializable()).unwrap();
    assert_eq!(variant.type_(), VariantTy::VARIANT);
    assert_eq!(variant.to_string(), "<@ms 'Hello'>");

    let variant = to_variant(Option::<String>::None.to_variant().as_serializable()).unwrap();
    assert_eq!(variant.type_(), VariantTy::VARIANT);
    assert_eq!(variant.to_string(), "<@ms nothing>");

    let variant = to_variant([1u32, 2u32, 3u32].to_variant().as_serializable()).unwrap();
    assert_eq!(variant.type_(), VariantTy::VARIANT);
    assert_eq!(variant.to_string(), "<[uint32 1, 2, 3]>");

    let variant = to_variant(("Hello", "World").to_variant().as_serializable()).unwrap();
    assert_eq!(variant.type_(), VariantTy::VARIANT);
    assert_eq!(variant.to_string(), "<('Hello', 'World')>");

    let variant = to_variant(
        HashMap::from([(1i64, "Hello")])
            .to_variant()
            .as_serializable(),
    )
    .unwrap();
    assert_eq!(variant.type_().as_str(), VariantTy::VARIANT);
    assert_eq!(variant.to_string(), "<{int64 1: 'Hello'}>");
}
