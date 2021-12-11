use glib::StaticVariantType;
use glib_serde::{from_variant, to_variant, Variant};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
#[derive(glib_serde::VariantType, serde::Serialize, serde::Deserialize)]
enum MyEnum {
    UnitVariant,
    NewtypeVariant(u16),
    TupleVariant(u8, String),
    StructVariant {
        data: Vec<i16>,
        keys: HashMap<i64, String>,
    },
}

#[derive(Debug, PartialEq, Eq)]
#[derive(glib_serde::VariantType, serde::Serialize, serde::Deserialize)]
struct MyNewtypeStruct(i32);

#[derive(Debug, PartialEq, Eq)]
#[derive(glib_serde::VariantType, serde::Serialize, serde::Deserialize)]
struct MyTupleStruct(u64, String, Option<String>);

#[derive(Debug, PartialEq)]
#[derive(glib_serde::VariantType, serde::Serialize, serde::Deserialize)]
struct MyStruct {
    id: u32,
    position: f64,
    my_tuple: MyTupleStruct,
    my_enum: MyEnum,
    my_enum2: Option<MyEnum>,
}

#[test]
fn serialize_enum() {
    assert_eq!(*MyEnum::static_variant_type(), "(sv)");

    let variant = to_variant(&MyEnum::UnitVariant).unwrap();
    assert_eq!(variant.type_(), "(sv)");
    assert_eq!(variant.child_value(1).as_variant().unwrap().type_(), "()");
    assert_eq!(variant.to_string(), "('UnitVariant', <()>)");

    let variant = to_variant(&MyEnum::NewtypeVariant(54)).unwrap();
    assert_eq!(variant.type_(), "(sv)");
    assert_eq!(variant.child_value(1).as_variant().unwrap().type_(), "q");
    assert_eq!(variant.to_string(), "('NewtypeVariant', <uint16 54>)");

    let variant = to_variant(&MyEnum::TupleVariant(8, "Eight".into())).unwrap();
    assert_eq!(variant.type_(), "(sv)");
    assert_eq!(variant.child_value(1).as_variant().unwrap().type_(), "(ys)");
    assert_eq!(variant.to_string(), "('TupleVariant', <(byte 0x08, 'Eight')>)");

    let variant = to_variant(&MyEnum::StructVariant {
        data: vec![3, 2, 1],
        keys: HashMap::from([(0, "Zero".into())]),
    }).unwrap();
    assert_eq!(variant.type_(), "(sv)");
    assert_eq!(variant.child_value(1).as_variant().unwrap().type_(), "(ana{xs})");
    assert_eq!(variant.to_string(), "('StructVariant', <([int16 3, 2, 1], {int64 0: 'Zero'})>)");
}

#[test]
fn serialize_structs() {
    assert_eq!(*MyNewtypeStruct::static_variant_type(), "i");
    let variant = to_variant(&MyNewtypeStruct(52)).unwrap();
    assert_eq!(variant.type_(), "i");
    assert_eq!(variant.to_string(), "52");

    assert_eq!(*MyTupleStruct::static_variant_type(), "(tsms)");
    let variant = to_variant(&MyTupleStruct(3, "hello".into(), Some("world".into()))).unwrap();
    assert_eq!(variant.type_(), "(tsms)");
    assert_eq!(variant.to_string(), "(3, 'hello', 'world')");

    assert_eq!(*MyStruct::static_variant_type(), "(ud(tsms)(sv)m(sv))");
    let variant = to_variant(&MyStruct {
        id: 3050,
        position: -182.5,
        my_tuple: MyTupleStruct(99, "Foo".into(), None),
        my_enum: MyEnum::StructVariant {
            data: vec![7, 6, 5],
            keys: HashMap::from([(-100, "Goodbye".into())]),
        },
        my_enum2: Some(MyEnum::UnitVariant),
    }).unwrap();
    assert_eq!(variant.type_(), "(ud(tsms)(sv)m(sv))");
    assert_eq!(
        variant.to_string(),
        "(\
            3050, \
            -182.5, \
            (99, 'Foo', nothing), \
            ('StructVariant', <([int16 7, 6, 5], {int64 -100: 'Goodbye'})>), \
            ('UnitVariant', <()>)\
        )"
    );
}

#[test]
fn deserialize_enums() {
    let s = "('UnitVariant', <()>)";
    let value: MyEnum = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, MyEnum::UnitVariant);

    let s = "('NewtypeVariant', <uint16 54>)";
    let value: MyEnum = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, MyEnum::NewtypeVariant(54));

    let s = "('TupleVariant', <(byte 0x08, 'Eight')>)";
    let value: MyEnum = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, MyEnum::TupleVariant(8, "Eight".into()));

    let s = "('StructVariant', <([int16 3, 2, 1], {int64 0: 'Zero'})>)";
    let value: MyEnum = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, MyEnum::StructVariant {
        data: vec![3, 2, 1],
        keys: HashMap::from([(0, "Zero".into())]),
    });
}

#[test]
fn deserialize_structs() {
    let s = "52";
    let value: MyNewtypeStruct = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, MyNewtypeStruct(52));

    let s = "(uint64 3, 'hello', just 'world')";
    let value: MyTupleStruct = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, MyTupleStruct(3, "hello".into(), Some("world".into())));

    let s = "(\
        uint32 3050, \
        -182.5, \
        (uint64 99, 'Foo', @ms nothing), \
        ('StructVariant', <([int16 7, 6, 5], {int64 -100: 'Goodbye'})>), \
        just ('UnitVariant', <()>)\
    )";
    let value: MyStruct = from_variant(&s.parse::<Variant>().unwrap()).unwrap();
    assert_eq!(value, MyStruct {
        id: 3050,
        position: -182.5,
        my_tuple: MyTupleStruct(99, "Foo".into(), None),
        my_enum: MyEnum::StructVariant {
            data: vec![7, 6, 5],
            keys: HashMap::from([(-100, "Goodbye".into())]),
        },
        my_enum2: Some(MyEnum::UnitVariant),
    });
}
