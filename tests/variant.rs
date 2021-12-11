// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use glib::{ToVariant, VariantTy};
use glib_serde::{
    from_variant, prelude::*, to_variant, ObjectPath, Signature, Variant, VariantDict,
};

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

#[test]
fn deserialize_basic_types() {
    let s = "true";
    let value: bool = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, true);

    let s = "int16 -3";
    let value: i16 = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, -3i16);

    let s = "-4000000";
    let value: i32 = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, -4000000i32);

    let s = "int64 -5000000000000";
    let value: i64 = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, -5000000000000i64);

    let s = "byte 0x06";
    let value: u8 = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, 6);

    let s = "uint16 7";
    let value: u16 = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, 7);

    let s = "uint32 8";
    let value: u32 = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, 8);

    let s = "uint64 9";
    let value: u64 = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, 9);

    let s = "10.1";
    let value: f64 = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, 10.1);

    let s = "'123'";
    let value: String = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, "123");

    let s = "objectpath '/com/org/Test'";
    let value: ObjectPath = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.as_str(), "/com/org/Test");

    let s = "signature '(asgva(in)a{sb})'";
    let value: Signature = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.as_str(), "(asgva(in)a{sb})");
}

#[test]
fn deserialize_container_types() {
    let s = "just 'Hello'";
    let value: Option<String> = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.as_ref().map(|s| s.as_str()), Some("Hello"));

    let s = "@ms nothing";
    let value: Option<String> = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, None);

    let s = "[1, 2, 3]";
    let value: Vec<i32> = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, [1, 2, 3]);

    let s = "{2: 'World'}";
    let value: HashMap<i32, String> = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, HashMap::from([(2, "World".into())]));

    let s = "{'b': <(int64 300, 400.5)>, 'a': <200>, 'c': <(int64 300, 500.5)>}";
    let value: VariantDict = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.lookup_value("a", None).unwrap().type_(), "i");
    assert_eq!(value.lookup_value("b", None).unwrap().type_(), "(xd)");
    assert_eq!(value.lookup_value("c", None).unwrap().type_(), "(xd)");

    let s = "<'Hello'>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "s");
    assert_eq!(value.str().unwrap(), "Hello");

    let s = "<objectpath '/com/org/Test'>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "o");
    assert_eq!(value.str().unwrap(), "/com/org/Test");

    let s = "<signature 'a(is)'>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "g");
    assert_eq!(value.str().unwrap(), "a(is)");

    let s = "<{'b': <('hello', 'world')>, 'a': <'abc'>}>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "a{sv}");
    let value = value.get::<VariantDict>().unwrap();
    assert_eq!(value.lookup_value("a", None).unwrap().type_(), "s");
    assert_eq!(value.lookup_value("b", None).unwrap().type_(), "(ss)");

    let s = "<@ms 'Hello'>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "ms");
    assert_eq!(value.maybe().flatten().unwrap().str().unwrap(), "Hello");

    let s = "<@ms nothing>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "ms");
    assert_eq!(value.maybe().unwrap(), None);

    let s = "<[uint32 1, 2, 3]>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "au");
    assert_eq!(value.fixed_array::<u32>().unwrap(), [1u32, 2, 3]);

    let s = "<('Hello', 'World')>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "(ss)");
    assert_eq!(
        value.get::<(String, String)>().unwrap(),
        ("Hello".into(), "World".into())
    );

    let s = "<{int64 1: 'Hello'}>";
    let value: Variant = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value.type_(), "a{xs}");
    assert_eq!(
        value.get::<HashMap<i64, String>>().unwrap(),
        HashMap::from([(1i64, "Hello".into())])
    );
}
