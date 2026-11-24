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

//! Minimal implementation of google.protobuf.compiler plugin messages for testing.
#![allow(dead_code)] // Each integration test crate uses only a subset of these helpers.
//!
//! This module provides just enough functionality to test the protoc plugin wrapper,
//! using protobuf-core for parsing and serialization.
//!
//! See: google/protobuf/compiler/plugin.proto in the Google Protobuf repository.
//!
//! ## Why a custom implementation?
//!
//! This crate uses `protobuf-core` instead of full protobuf libraries (prost, protobuf, etc.)
//! to minimize dependencies. The test helper only implements the minimal fields needed
//! for testing (`CodeGeneratorRequest.proto_file` count, `CodeGeneratorResponse.file` list).

use ::protobuf_core::{AsRefExtProtobuf, Field, FieldNumber, FieldValue, WriteExtProtobuf};
use ::std::io::Write;

// Field numbers from google/protobuf/compiler/plugin.proto
const CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER: u32 = 2;
const CODE_GENERATOR_REQUEST_PROTO_FILE_FIELD_NUMBER: u32 = 15;
const CODE_GENERATOR_RESPONSE_FILE_FIELD_NUMBER: u32 = 15;
const FILE_NAME_FIELD_NUMBER: u32 = 1;
const FILE_CONTENT_FIELD_NUMBER: u32 = 15;

/// Minimal implementation of google.protobuf.compiler.CodeGeneratorRequest
///
/// This only implements the fields needed for testing:
/// - `proto_file` (field 15): The `FileDescriptorProto` messages
#[derive(Debug, Default)]
pub struct CodeGeneratorRequest {
    /// Repeated `FileDescriptorProto` `proto_file` = 15;
    /// For testing, we just count them, not parse the full `FileDescriptorProto`
    pub proto_file_count: usize,
}

impl CodeGeneratorRequest {
    /// Parse a `CodeGeneratorRequest` from bytes
    pub fn from_bytes(bytes: &[u8]) -> protobuf_core::Result<Self> {
        let mut proto_file_count = 0;

        for field_result in bytes.read_protobuf_fields() {
            let field = field_result?;

            if field.field_number.as_u32() == CODE_GENERATOR_REQUEST_PROTO_FILE_FIELD_NUMBER {
                // Each Len field is one FileDescriptorProto (we don't parse it, just count)
                if matches!(field.value, FieldValue::Len(_)) {
                    proto_file_count += 1;
                }
            }
            // Ignore other fields for testing purposes
        }

        Ok(Self { proto_file_count })
    }
}

/// Last Len UTF-8 string for `target_field_number` (last-wins), or `None`.
pub fn last_string_field(
    bytes: &[u8],
    target_field_number: u32,
) -> protobuf_core::Result<Option<String>> {
    let mut result = None;
    for field_result in bytes.read_protobuf_fields() {
        let field = field_result?;
        if field.field_number.as_u32() == target_field_number
            && let FieldValue::Len(payload) = field.value
        {
            result = Some(
                ::std::str::from_utf8(payload)
                    .expect("Len string field must be UTF-8")
                    .to_owned(),
            );
        }
    }
    Ok(result)
}

/// `CodeGeneratorRequest.parameter` (field 2) — the IPC oneshot server name.
pub fn request_parameter(bytes: &[u8]) -> protobuf_core::Result<Option<String>> {
    last_string_field(bytes, CODE_GENERATOR_REQUEST_PARAMETER_FIELD_NUMBER)
}

/// Minimal implementation of google.protobuf.compiler.CodeGeneratorResponse.File
///
/// This implements:
/// - name (field 1): The output file name
/// - content (field 15): The file content
#[derive(Debug, Default, Clone)]
pub struct File {
    /// optional string name = 1;
    pub name: String,
    /// optional string content = 15;
    pub content: String,
}

/// Minimal implementation of google.protobuf.compiler.CodeGeneratorResponse
///
/// This only implements:
/// - file (field 15): The generated files
#[derive(Debug, Default)]
pub struct CodeGeneratorResponse {
    /// repeated File file = 15;
    pub files: Vec<File>,
}

impl CodeGeneratorResponse {
    /// Serialize the response to bytes
    pub fn to_bytes(&self, writer: &mut impl Write) -> protobuf_core::Result<usize> {
        let mut total_bytes = 0;

        for file in &self.files {
            // Serialize each file as a length-delimited message
            let mut file_bytes = Vec::new();

            // Write name field if present
            if !file.name.is_empty() {
                let field = Field::new(
                    FieldNumber::try_from(FILE_NAME_FIELD_NUMBER).unwrap(),
                    FieldValue::Len(file.name.as_bytes()),
                );
                file_bytes.write_protobuf_field(&field)?;
            }

            // Write content field if present
            if !file.content.is_empty() {
                let field = Field::new(
                    FieldNumber::try_from(FILE_CONTENT_FIELD_NUMBER).unwrap(),
                    FieldValue::Len(file.content.as_bytes()),
                );
                file_bytes.write_protobuf_field(&field)?;
            }

            // Write the file as a length-delimited field (field 15)
            let response_field = Field::new(
                FieldNumber::try_from(CODE_GENERATOR_RESPONSE_FILE_FIELD_NUMBER).unwrap(),
                FieldValue::Len(file_bytes),
            );
            total_bytes += writer.write_protobuf_field(&response_field)?;
        }

        Ok(total_bytes)
    }
}

/// Encode a one-file `CodeGeneratorResponse`.
pub fn response_with_files(files: &[(&str, &str)]) -> Vec<u8> {
    let mut res = CodeGeneratorResponse::default();
    for &(name, content) in files {
        res.files.push(File {
            name: name.to_string(),
            content: content.to_string(),
        });
    }
    let mut bytes = Vec::new();
    res.to_bytes(&mut bytes).unwrap();
    bytes
}

/// Encode a single-file response.
pub fn response_with_one_file(name: &str, content: &str) -> Vec<u8> {
    response_with_files(&[(name, content)])
}

/// Assert the request has exactly one `proto_file`, then return a one-file response.
pub fn assert_one_proto_file_and_respond(
    req_bytes: &[u8],
    out_file_name: &str,
    out_file_content: &str,
) -> Vec<u8> {
    let req = CodeGeneratorRequest::from_bytes(req_bytes).unwrap();
    assert_eq!(req.proto_file_count, 1);
    response_with_one_file(out_file_name, out_file_content)
}
