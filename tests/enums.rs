// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    glib::Enum,
    glib_serde::EnumSerialize,
    glib_serde::EnumDeserialize,
)]
#[enum_type(name = "MyEnum")]
enum MyEnum {
    Val,
    #[enum_value(name = "My Val")]
    ValWithCustomName,
    #[enum_value(name = "My Other Val", nick = "other")]
    ValWithCustomNameAndNick,
}

#[test]
fn serialize_enum() {
    let json = serde_json::to_string(&MyEnum::ValWithCustomName).unwrap();
    assert_eq!(json, "\"val-with-custom-name\"");

    let json = serde_json::to_string(&MyEnum::ValWithCustomNameAndNick).unwrap();
    assert_eq!(json, "\"other\"");
}

#[test]
fn deserialize_enum() {
    let e: MyEnum = serde_json::from_str("\"val-with-custom-name\"").unwrap();
    assert_eq!(e, MyEnum::ValWithCustomName);

    let e: MyEnum = serde_json::from_str("\"other\"").unwrap();
    assert_eq!(e, MyEnum::ValWithCustomNameAndNick);

    let err = serde_json::from_str::<'_, MyEnum>("\"nothing\"").unwrap_err();
    assert!(err
        .to_string()
        .contains("expected a valid enum value for MyEnum"));
}

#[derive(glib_serde::FlagsSerialize, glib_serde::FlagsDeserialize)]
#[glib::flags(name = "MyFlags")]
enum MyFlags {
    #[flags_value(name = "Flag A", nick = "nick-a")]
    A = 0b00000001,
    #[flags_value(name = "Flag B")]
    B = 0b00000010,
    #[flags_value(skip)]
    AB = Self::A.bits() | Self::B.bits(),
    C = 0b00000100,
}

#[test]
fn serialize_flags() {
    let json = serde_json::to_string(&MyFlags::empty()).unwrap();
    assert_eq!(json, "\"\"");

    let json = serde_json::to_string(&MyFlags::A).unwrap();
    assert_eq!(json, "\"nick-a\"");

    let json = serde_json::to_string(&MyFlags::B).unwrap();
    assert_eq!(json, "\"b\"");

    let json = serde_json::to_string(&(MyFlags::B | MyFlags::C)).unwrap();
    assert_eq!(json, "\"b|c\"");

    let json = serde_json::to_string(&MyFlags::AB).unwrap();
    assert_eq!(json, "\"nick-a|b\"");

    let json = serde_json::to_string(&MyFlags::all()).unwrap();
    assert_eq!(json, "\"nick-a|b|c\"");
}

#[test]
fn deserialize_flags() {
    let f: MyFlags = serde_json::from_str("\"\"").unwrap();
    assert_eq!(f, MyFlags::empty());

    let f: MyFlags = serde_json::from_str("\"nick-a\"").unwrap();
    assert_eq!(f, MyFlags::A);

    let f: MyFlags = serde_json::from_str("\"nick-a|b\"").unwrap();
    assert_eq!(f, MyFlags::AB);

    let err = serde_json::from_str::<'_, MyFlags>("\"nick-a|b|bad|c\"").unwrap_err();
    assert!(err
        .to_string()
        .contains("expected a valid flags value for MyFlags"));
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    glib::Enum,
    glib_serde::EnumSerialize,
    glib_serde::EnumDeserialize,
)]
#[enum_type(name = "MyEnum2")]
#[glib_serde_repr]
enum MyEnum2 {
    Val,
    #[enum_value(name = "My Val")]
    ValWithCustomName,
    #[enum_value(name = "My Other Val", nick = "other")]
    ValWithCustomNameAndNick,
}

#[derive(glib_serde::FlagsSerialize, glib_serde::FlagsDeserialize)]
#[glib::flags(name = "MyFlags2")]
#[glib_serde_repr]
enum MyFlags2 {
    #[flags_value(name = "Flag A", nick = "nick-a")]
    A = 0b00000001,
    #[flags_value(name = "Flag B")]
    B = 0b00000010,
    #[flags_value(skip)]
    AB = Self::A.bits() | Self::B.bits(),
    C = 0b00000100,
}

#[test]
fn serialize_repr() {
    let json = serde_json::to_string(&MyEnum2::ValWithCustomName).unwrap();
    assert_eq!(json, "1");

    let json = serde_json::to_string(&MyFlags2::empty()).unwrap();
    assert_eq!(json, "0");

    let json = serde_json::to_string(&MyFlags2::AB).unwrap();
    assert_eq!(json, "3");
}

#[test]
fn deserialize_repr() {
    let e: MyEnum2 = serde_json::from_str("2").unwrap();
    assert_eq!(e, MyEnum2::ValWithCustomNameAndNick);

    let f: MyFlags2 = serde_json::from_str("0").unwrap();
    assert_eq!(f, MyFlags2::empty());

    let f: MyFlags2 = serde_json::from_str("1").unwrap();
    assert_eq!(f, MyFlags2::A);

    let f: MyFlags2 = serde_json::from_str("3").unwrap();
    assert_eq!(f, MyFlags2::AB);
}
