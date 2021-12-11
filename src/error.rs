// SPDX-FileCopyrightText: 2021 Jason Francis <jafrancis999@gmail.com>
// SPDX-License-Identifier: MIT

use std::{fmt::Display, num::TryFromIntError};

use glib::{variant::VariantTypeMismatchError, BoolError};

/// Error type for deserialization and serialization.
#[derive(Debug)]
pub enum Error {
    Bool(BoolError),
    Mismatch(VariantTypeMismatchError),
    Int(TryFromIntError),
    StrMismatch(glib::VariantType),
    InvalidTag(glib::VariantType),
    UnsupportedType(glib::VariantType),
    ExpectedChar(String),
    InvalidType(String),
    LengthMismatch { actual: usize, expected: usize },
    Custom(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(e) => e.fmt(f),
            Self::Mismatch(e) => e.fmt(f),
            Self::Int(e) => e.fmt(f),
            Self::StrMismatch(actual) => {
                write!(
                    f,
                    "Type mismatch: Expected 's', 'o', or 'g', got '{}'",
                    actual
                )
            }
            Self::InvalidTag(actual) => {
                write!(f, "Invalid enum tag type: '{}'", actual)
            }
            Self::UnsupportedType(actual) => {
                write!(f, "Type not supported: '{}'", actual)
            }
            Self::ExpectedChar(s) => {
                write!(
                    f,
                    "Type mismatch: Expected string with length 1, got '{}'",
                    s
                )
            }
            Self::InvalidType(s) => {
                write!(f, "Invalid GVariant type string: {}", s)
            }
            Self::LengthMismatch { actual, expected } => {
                write!(
                    f,
                    "Struct/tuple length mismatch: Expected {}, got {}",
                    expected, actual
                )
            }
            Self::Custom(e) => e.fmt(f),
        }
    }
}

impl From<BoolError> for Error {
    fn from(e: BoolError) -> Self {
        Self::Bool(e)
    }
}

impl From<VariantTypeMismatchError> for Error {
    fn from(e: VariantTypeMismatchError) -> Self {
        Self::Mismatch(e)
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Self::Int(e)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bool(e) => Some(e),
            Self::Mismatch(e) => Some(e),
            Self::Int(e) => Some(e),
            _ => None,
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}
