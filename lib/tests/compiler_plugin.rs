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
//!
//! This module provides just enough functionality to test the protoc plugin wrapper,
//! using protobuf-core for parsing and serialization.
//!
//! See: google/protobuf/compiler/plugin.proto in the Google Protobuf repository.

use ::protobuf_core::{Field, FieldNumber, FieldValue, ReadExtProtobuf, WriteExtProtobuf};
use ::std::io::{Error as IoError, ErrorKind, Result as IoResult, Write};

// Field numbers from google/protobuf/compiler/plugin.proto
const CODE_GENERATOR_REQUEST_PROTO_FILE_FIELD_NUMBER: u32 = 15;
const CODE_GENERATOR_RESPONSE_FILE_FIELD_NUMBER: u32 = 15;
const FILE_NAME_FIELD_NUMBER: u32 = 1;
const FILE_CONTENT_FIELD_NUMBER: u32 = 15;

/// Minimal implementation of google.protobuf.compiler.CodeGeneratorRequest
///
/// This only implements the fields needed for testing:
/// - proto_file (field 15): The FileDescriptorProto messages
#[derive(Debug, Default)]
pub struct CodeGeneratorRequest {
    /// Repeated FileDescriptorProto proto_file = 15;
    /// For testing, we just count them, not parse the full FileDescriptorProto
    pub proto_file_count: usize,
}

impl CodeGeneratorRequest {
    /// Parse a CodeGeneratorRequest from bytes
    pub fn from_bytes(bytes: &[u8]) -> IoResult<Self> {
        let mut proto_file_count = 0;

        for field_result in bytes.read_protobuf_fields() {
            let field = field_result.map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;

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
    pub fn to_bytes(&self, writer: &mut impl Write) -> IoResult<usize> {
        let mut total_bytes = 0;

        for file in &self.files {
            // Serialize each file as a length-delimited message
            let mut file_bytes = Vec::new();

            // Write name field if present
            if !file.name.is_empty() {
                let field = Field::new(
                    FieldNumber::try_from(FILE_NAME_FIELD_NUMBER).unwrap(),
                    FieldValue::from_string(file.name.clone()),
                );
                file_bytes
                    .write_protobuf_field(&field)
                    .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;
            }

            // Write content field if present
            if !file.content.is_empty() {
                let field = Field::new(
                    FieldNumber::try_from(FILE_CONTENT_FIELD_NUMBER).unwrap(),
                    FieldValue::from_string(file.content.clone()),
                );
                file_bytes
                    .write_protobuf_field(&field)
                    .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;
            }

            // Write the file as a length-delimited field (field 15)
            let response_field = Field::new(
                FieldNumber::try_from(CODE_GENERATOR_RESPONSE_FILE_FIELD_NUMBER).unwrap(),
                FieldValue::from_bytes(file_bytes),
            );
            total_bytes += writer
                .write_protobuf_field(&response_field)
                .map_err(|e| IoError::new(ErrorKind::InvalidData, e))?;
        }

        Ok(total_bytes)
    }
}
