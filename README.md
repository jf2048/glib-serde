# glib-serde

## Serde support for GLib types in gtk-rs-core

Supports serializing arbitrary types to/from `glib::Variant` using
[serde](https://serde.rs). The main interface is `to_variant` and `from_variant`.

Serializing structs and enums requires an implementation of `VariantType`, which should be
automatically derived:

```rust
#[derive(Debug, PartialEq, Eq)]
#[derive(glib_serde::VariantType, serde::Serialize, serde::Deserialize)]
struct MyStruct {
    id: i32,
    name: String
}

let s = MyStruct {
    id: 1,
    name: String::from("Item")
};
let variant = glib_serde::to_variant(&s).unwrap();
assert_eq!(variant.type_(), "(is)");
assert_eq!(variant.to_string(), "(1, 'Item')");
let value: MyStruct = glib_serde::from_variant(&variant).unwrap();
assert_eq!(s, value);
```

Additional derive macros are provided to serialize/deserialize GLib enum and flag types:

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, glib::Enum)]
#[derive(glib_serde::VariantType, glib_serde::EnumSerialize, glib_serde::EnumDeserialize)]
#[enum_type(name = "Direction")]
enum Direction {
    North = 1,
    East = 2,
    South = 3,
    West = 4,
}

let variant = glib_serde::to_variant(&Direction::South).unwrap();
assert_eq!(variant.type_(), "s");
assert_eq!(variant.to_string(), "'south'");
let value: Direction = glib_serde::from_variant(&variant).unwrap();
assert_eq!(value, Direction::South);
```
