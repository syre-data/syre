use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Value {
    /// Empty data.
    Null,
    Bool(bool),
    String(String),

    /// Pure number.
    Number(serde_json::Number),

    /// Magnitude with unit.
    Quantity {
        magnitude: f64,
        unit: String,
    },

    Array(Vec<Self>),
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Null => todo!(),
            Value::Bool(_) => ValueKind::Bool,
            Value::String(_) => ValueKind::String,
            Value::Number(_) => ValueKind::Number,
            Value::Quantity { .. } => ValueKind::Quantity,
            Value::Array(_) => ValueKind::Array,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "(null)"),
            Value::Bool(value) => write!(f, "{value}"),
            Value::String(value) => write!(f, r#""{value}""#),
            Value::Number(number) => write!(f, "{number}"),
            Value::Quantity { magnitude, unit } => write!(f, "{magnitude} {unit}"),
            Value::Array(vec) => write!(f, "{vec:?}"),
        }
    }
}

// Implementing Eq is fine because float values are always finite.
impl Eq for Value {}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ValueKind {
    Bool,
    String,
    Number,
    Quantity,
    Array,
}

mod from {
    //! Taken from `serde_json`.
    //! See [https://github.com/serde-rs/json/blob/master/src/value/from.rs].
    use super::Value;
    use std::borrow::Cow;
    use std::string::{String, ToString};
    use std::vec::Vec;

    macro_rules! from_integer {
        ($($ty:ident)*) => {
            $(
                impl From<$ty> for Value {
                    fn from(n: $ty) -> Self {
                        Value::Number(n.into())
                    }
                }
            )*
        };
    }

    from_integer! {
        i8 i16 i32 i64 isize
        u8 u16 u32 u64 usize
    }

    fn from_f32(f: f32) -> Option<serde_json::Number> {
        serde_json::Number::from_f64(f as f64)
    }

    impl From<f32> for Value {
        /// Convert 32-bit floating point number to `Value::Number`, or
        /// `Value::Null` if infinite or NaN.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let f: f32 = 13.37;
        /// let x: Value = f.into();
        /// ```
        fn from(f: f32) -> Self {
            from_f32(f).map_or(Value::Null, Value::Number)
        }
    }

    impl From<f64> for Value {
        /// Convert 64-bit floating point number to `Value::Number`, or
        /// `Value::Null` if infinite or NaN.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let f: f64 = 13.37;
        /// let x: Value = f.into();
        /// ```
        fn from(f: f64) -> Self {
            serde_json::Number::from_f64(f).map_or(Value::Null, Value::Number)
        }
    }

    impl From<bool> for Value {
        /// Convert boolean to `Value::Bool`.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let b = false;
        /// let x: Value = b.into();
        /// ```
        fn from(f: bool) -> Self {
            Value::Bool(f)
        }
    }

    impl From<String> for Value {
        /// Convert `String` to `Value::String`.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let s: String = "lorem".to_string();
        /// let x: Value = s.into();
        /// ```
        fn from(f: String) -> Self {
            Value::String(f)
        }
    }

    impl From<&str> for Value {
        /// Convert string slice to `Value::String`.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let s: &str = "lorem";
        /// let x: Value = s.into();
        /// ```
        fn from(f: &str) -> Self {
            Value::String(f.to_string())
        }
    }

    impl<'a> From<Cow<'a, str>> for Value {
        /// Convert copy-on-write string to `Value::String`.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        /// use std::borrow::Cow;
        ///
        /// let s: Cow<str> = Cow::Borrowed("lorem");
        /// let x: Value = s.into();
        /// ```
        ///
        /// ```
        /// use syre_core::types::Value;
        /// use std::borrow::Cow;
        ///
        /// let s: Cow<str> = Cow::Owned("lorem".to_string());
        /// let x: Value = s.into();
        /// ```
        fn from(f: Cow<'a, str>) -> Self {
            Value::String(f.into_owned())
        }
    }

    impl From<serde_json::Number> for Value {
        /// Convert `Number` to `Value::Number`.
        ///
        /// # Examples
        ///
        /// ```
        /// use serde_json::Number;
        /// use syre_core::types::Value;
        ///
        /// let n = Number::from(7);
        /// let x: Value = n.into();
        /// ```
        fn from(f: serde_json::Number) -> Self {
            Value::Number(f)
        }
    }

    impl<T: Into<Value>> From<Vec<T>> for Value {
        /// Convert a `Vec` to `Value::Array`.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let v = vec!["lorem", "ipsum", "dolor"];
        /// let x: Value = v.into();
        /// ```
        fn from(f: Vec<T>) -> Self {
            Value::Array(f.into_iter().map(Into::into).collect())
        }
    }

    impl<T: Clone + Into<Value>> From<&[T]> for Value {
        /// Convert a slice to `Value::Array`.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let v: &[&str] = &["lorem", "ipsum", "dolor"];
        /// let x: Value = v.into();
        /// ```
        fn from(f: &[T]) -> Self {
            Value::Array(f.iter().cloned().map(Into::into).collect())
        }
    }

    impl<T: Into<Value>> FromIterator<T> for Value {
        /// Create a `Value::Array` by collecting an iterator of array elements.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let v = std::iter::repeat(42).take(5);
        /// let x: Value = v.collect();
        /// ```
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let v: Vec<_> = vec!["lorem", "ipsum", "dolor"];
        /// let x: Value = v.into_iter().collect();
        /// ```
        ///
        /// ```
        /// use std::iter::FromIterator;
        /// use syre_core::types::Value;
        ///
        /// let x: Value = Value::from_iter(vec!["lorem", "ipsum", "dolor"]);
        /// ```
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            Value::Array(iter.into_iter().map(Into::into).collect())
        }
    }

    impl From<()> for Value {
        /// Convert `()` to `Value::Null`.
        ///
        /// # Examples
        ///
        /// ```
        /// use syre_core::types::Value;
        ///
        /// let u = ();
        /// let x: Value = u.into();
        /// ```
        fn from((): ()) -> Self {
            Value::Null
        }
    }

    impl<T> From<Option<T>> for Value
    where
        T: Into<Value>,
    {
        fn from(opt: Option<T>) -> Self {
            match opt {
                None => Value::Null,
                Some(value) => Into::into(value),
            }
        }
    }

    impl From<serde_json::Value> for Value {
        fn from(value: serde_json::Value) -> Self {
            match value {
                serde_json::Value::Null => Value::Null,
                serde_json::Value::Bool(b) => Value::Bool(b),
                serde_json::Value::Number(n) => Value::Number(n),
                serde_json::Value::String(s) => Value::String(s),
                serde_json::Value::Array(arr) => {
                    let arr = arr.into_iter().map(|elm| elm.into()).collect();
                    Value::Array(arr)
                }
                serde_json::Value::Object(_obj) => {
                    todo!("map is an invalid kind, probably need to return `Result` here")
                }
            }
        }
    }
}
