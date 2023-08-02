//! Common functions to metadata.
use super::MetadatumType;
use serde_json::{Result as JsResult, Value as JsValue};
use std::result::Result as StdResult;
use std::str::FromStr;
use yew::prelude::NodeRef;

/// Converts a string to a number.
/// Is restrictive as possible in conversion.
/// i.e. First tries to convert to `u64`, then `i64`, then `f64`.
///
/// # Returns
/// A [`serde_json::Value`] that is a
/// + [`Number`](serde_json::value::Number) if the value is finite and parsed correctly.
/// + `Null` if the value is parsed correclty but `nan`.
/// + 0 if the value is empty. (This also occurs if the string is an invalid number.)
///
/// # Errors
/// + If the value can not be parsed as a number.
#[tracing::instrument]
pub fn str_to_number(input: &str) -> StdResult<JsValue, <f64 as FromStr>::Err> {
    tracing::debug!(?input);
    if input.is_empty() {
        tracing::debug!("empty");
        return Ok(JsValue::from(0 as u64));
    }

    if let Ok(val) = input.parse::<u64>() {
        tracing::debug!("u");
        return Ok(JsValue::from(val));
    }

    if let Ok(val) = input.parse::<i64>() {
        tracing::debug!("i");
        return Ok(JsValue::from(val));
    }

    let val = input.parse::<f64>()?;
    tracing::debug!(?val);
    match val.is_nan() {
        true => Ok(JsValue::Null),
        false => Ok(JsValue::from(val)),
    }
}

/// # Returns
/// The input value converted to a [`serde_json::Value`].
/// `Null` is used to indicate an error occurred.
#[tracing::instrument(skip(value_ref))]
pub fn value_from_input(value_ref: NodeRef, kind: &MetadatumType) -> JsResult<JsValue> {
    let value = match kind {
        MetadatumType::String => {
            let v_in = value_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not convert value node ref into input");

            let val = v_in.value().trim().to_owned();
            match val.is_empty() {
                true => JsValue::Null,
                false => JsValue::String(val),
            }
        }

        MetadatumType::Number => {
            let v_in = value_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not convert value node ref into input");

            match str_to_number(v_in.value().trim()) {
                Ok(val) => val,
                Err(_) => JsValue::Null,
            }
        }

        MetadatumType::Bool => {
            let v_in = value_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not convert value node ref into input");

            JsValue::Bool(v_in.checked())
        }

        MetadatumType::Array => {
            let v_in = value_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .expect("could not cast value node ref as textarea");

            let val = v_in.value().trim().to_owned();
            match val.is_empty() {
                true => JsValue::Null,
                false => serde_json::from_str(&val)?,
            }
        }

        MetadatumType::Object => {
            let v_in = value_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .expect("could not cast value node ref as textarea");

            let val = v_in.value().trim().to_owned();
            match val.is_empty() {
                true => JsValue::Null,
                false => serde_json::from_str(&val)?,
            }
        }
    };

    Ok(value)
}

/// Converts between [`serde_json::Value`] types.
/// If a reasonable conversion can not be made, the default value for that type is returned.
#[tracing::instrument]
pub fn convert_value(value: JsValue, target: &MetadatumType) -> JsValue {
    match (value.clone(), target.clone()) {
        (JsValue::String(_), MetadatumType::String)
        | (JsValue::Number(_), MetadatumType::Number)
        | (JsValue::Bool(_), MetadatumType::Bool)
        | (JsValue::Array(_), MetadatumType::Array)
        | (JsValue::Object(_), MetadatumType::Object) => value,

        (JsValue::String(value), MetadatumType::Number) => match str_to_number(&value) {
            Ok(val) => val,
            Err(_) => JsValue::from(0 as u64),
        },

        (JsValue::Number(value), MetadatumType::String) => value.to_string().into(),

        (JsValue::Array(value), MetadatumType::String) => serde_json::to_string_pretty(&value)
            .unwrap_or(String::default())
            .into(),

        (JsValue::Object(value), MetadatumType::String) => serde_json::to_string_pretty(&value)
            .unwrap_or(String::default())
            .into(),

        (JsValue::String(value), MetadatumType::Array) => {
            let value = serde_json::to_value(value).unwrap_or_default();
            if value.is_array() {
                value
            } else {
                JsValue::Array(Vec::default())
            }
        }

        (JsValue::String(value), MetadatumType::Object) => {
            let value = serde_json::to_value(value).unwrap_or_default();
            if value.is_object() {
                value
            } else {
                JsValue::Object(serde_json::Map::default())
            }
        }

        (_, MetadatumType::String) => JsValue::String(String::default()),
        (_, MetadatumType::Number) => JsValue::Number(0.into()),
        (_, MetadatumType::Bool) => JsValue::Bool(false),
        (_, MetadatumType::Array) => JsValue::Array(Vec::default()),
        (_, MetadatumType::Object) => JsValue::Object(serde_json::Map::default()),
    }
}
