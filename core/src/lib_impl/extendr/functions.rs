//! Common functions.
use extendr_api::prelude::*;
use serde_json::value::{Number as JsNumber, Value as JsValue};

/// Convert an [`Robj`] to a [`Value`](JsValue).
/// Returns `None` if the conversion fails.
pub fn robj_to_value(obj: Robj) -> Option<JsValue> {
    if let Some(val) = obj.as_bool() {
        return Some(JsValue::Bool(val));
    } else if let Some(val) = obj.as_integer() {
        return Some(JsValue::Number(val.into()));
    } else if let Some(val) = obj.as_integer_vector() {
        let vals = val.into_iter().map(|n| JsValue::Number(n.into())).collect();
        return Some(JsValue::Array(vals));
    } else if let Some(val) = obj.as_real() {
        let val = JsNumber::from_f64(val)
            .expect("could not convert value to a serde_json::value::Number");

        return Some(JsValue::Number(val));
    } else if let Some(val) = obj.as_real_vector() {
        let vals = val
            .into_iter()
            .map(|n| {
                let n = JsNumber::from_f64(n)
                    .expect("could not convert value to a serde_json::value::Number");

                JsValue::Number(n)
            })
            .collect();

        return Some(JsValue::Array(vals));
    } else if let Some(val) = obj.as_str() {
        return Some(JsValue::String(val.into()));
    } else if let Some(val) = obj.as_string_vector() {
        let val = val.into_iter().map(|s| JsValue::String(s.into())).collect();
        return Some(JsValue::Array(val));
    }

    return None;
}
