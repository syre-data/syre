//! Result and Errors.
use gloo_storage::errors::StorageError as GlooStorageError;
use serde_wasm_bindgen as swb;
use std::result::Result as StdResult;
use wasm_bindgen::JsValue;

/// Errors
#[derive(Debug)]
pub enum Error {
    Serde(String),
    Binding(String),
    StorageError(GlooStorageError),
}

impl From<GlooStorageError> for Error {
    fn from(err: GlooStorageError) -> Self {
        Self::StorageError(err)
    }
}

impl From<swb::Error> for Error {
    fn from(err: swb::Error) -> Self {
        Self::Serde(err.to_string())
    }
}

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        Self::Binding(format!("{:?}", err))
    }
}

// *************
// *** serde ***
// *************

// use serde::de;
// use std::fmt;
//
// @todo[0]: Actually deserialize.
//      Currently all errors are turned into a string and shoved into an `InvokeError`.
// const VARIANTS: &'static [&'static str] = &["InvokeError", "StorageError"];

// struct ErrorVisitor;

// impl<'de> de::Visitor<'de> for ErrorVisitor {
//     type Value = Error;

//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("an error")
//     }

//     fn visit_map<A>(self, mut map: A) -> StdResult<Self::Value, A::Error>
//     where
//         A: de::MapAccess<'de>,
//     {
//         let mut out = String::new();
//         while let Some(key) = map.next_key::<String>()? {
//             let value = map.next_value()?;
//             out.push_str(&format!("{key}: {:#?},", value));
//         }

//         Ok(Error::InvokeError(out))
//     }

//     //    fn visit_enum<A>(self, data: A) -> StdResult<Self::Value, A::Error>
//     //    where
//     //        A: de::EnumAccess<'de>,
//     //    {
//     //        match data {
//     //            _ => Err(de::Error::unknown_variant(data, VARIANTS)),
//     //        }
//     //    }
// }

// impl<'de> de::Deserialize<'de> for Error {
//     fn deserialize<D>(deserializer: D) -> StdResult<Error, D::Error>
//     where
//         D: de::Deserializer<'de>,
//     {
//         deserializer.deserialize_enum("Error", VARIANTS, ErrorVisitor)
//     }
// }

// **************
// *** result ***
// **************

/// Crate result.
pub type Result<T = ()> = StdResult<T, Error>;
