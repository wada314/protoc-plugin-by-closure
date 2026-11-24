// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Extract `CodeGeneratorRequest.parameter` (field 2) from raw request bytes.

use ::anyhow::{Result, anyhow};
use ::protobuf_core::{AsRefExtProtobuf, FieldValue};

/// Field number for `CodeGeneratorRequest.parameter`.
///
/// See: `google/protobuf/compiler/plugin.proto` in the Google Protobuf repository.
pub const CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER: u32 = 2;

/// Returns the last Len string with `target_field_number`, or `Ok(None)`.
///
/// Duplicate occurrences use last-wins (protobuf merge semantics for singular
/// strings). Non-Len wire types for that number are ignored. Invalid UTF-8 in a
/// matching Len payload is an error.
pub fn find_last_string_field(input: &[u8], target_field_number: u32) -> Result<Option<String>> {
    let mut result: Option<String> = None;

    for field_result in input.read_protobuf_fields() {
        let field = field_result.map_err(|e| anyhow!("Failed to parse protobuf field: {}", e))?;

        if field.field_number.as_u32() == target_field_number
            && let FieldValue::Len(bytes) = field.value
        {
            result = Some(std::str::from_utf8(bytes)?.to_owned());
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER, find_last_string_field};
    use ::protobuf_core::{Field, FieldNumber, FieldValue, WriteExtProtobuf};

    fn encode_string_field(field_number: u32, value: &str) -> Vec<u8> {
        let mut out = Vec::new();
        let field = Field::new(
            FieldNumber::try_from(field_number).unwrap(),
            FieldValue::Len(value.as_bytes()),
        );
        out.write_protobuf_field(&field).unwrap();
        out
    }

    fn encode_varint_field(field_number: u32, value: u64) -> Vec<u8> {
        let mut out = Vec::new();
        let field: Field<&[u8]> = Field::new(
            FieldNumber::try_from(field_number).unwrap(),
            FieldValue::from_uint64(value),
        );
        out.write_protobuf_field(&field).unwrap();
        out
    }

    #[test]
    fn empty_input_returns_none() {
        assert_eq!(
            find_last_string_field(&[], CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER).unwrap(),
            None
        );
    }

    #[test]
    fn missing_field_returns_none() {
        let bytes = encode_string_field(15, "other");
        assert_eq!(
            find_last_string_field(&bytes, CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER).unwrap(),
            None
        );
    }

    #[test]
    fn single_parameter_is_returned() {
        let bytes = encode_string_field(CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER, "ipc-key");
        assert_eq!(
            find_last_string_field(&bytes, CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER)
                .unwrap()
                .as_deref(),
            Some("ipc-key")
        );
    }

    #[test]
    fn duplicate_parameter_last_wins() {
        let mut bytes = encode_string_field(CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER, "first");
        bytes.extend_from_slice(&encode_string_field(
            CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER,
            "second",
        ));
        assert_eq!(
            find_last_string_field(&bytes, CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER)
                .unwrap()
                .as_deref(),
            Some("second")
        );
    }

    #[test]
    fn non_len_wire_type_for_target_is_ignored() {
        let mut bytes = encode_varint_field(CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER, 1);
        bytes.extend_from_slice(&encode_string_field(
            CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER,
            "ok",
        ));
        assert_eq!(
            find_last_string_field(&bytes, CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER)
                .unwrap()
                .as_deref(),
            Some("ok")
        );
    }

    #[test]
    fn non_utf8_len_is_error() {
        let mut out = Vec::new();
        let field = Field::new(
            FieldNumber::try_from(CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER).unwrap(),
            FieldValue::Len(&[0xff, 0xfe]),
        );
        out.write_protobuf_field(&field).unwrap();
        assert!(
            find_last_string_field(&out, CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER).is_err()
        );
    }

    #[test]
    fn truncated_varint_is_error() {
        // Tag for field 2 Len (0x12) then a truncated length varint.
        let bytes = [0x12u8, 0x80];
        assert!(
            find_last_string_field(&bytes, CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER).is_err()
        );
    }
}
